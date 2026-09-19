/** Checks GitHub for a newer release, and installs it where the platform allows. */

import { useCallback, useEffect, useRef, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { canSelfInstall, openReleasesPage } from "./api";

/** Give the app time to settle before the first network call of the session. */
const STARTUP_CHECK_DELAY_MS = 8000;

export type UpdateState =
  | "idle"
  | "checking"
  | "available"
  | "none"
  | "error"
  | "downloading"
  | "installed";

export interface UpdateInfo {
  version: string;
  body: string | null;
  date: string | null;
}

export interface UseUpdateCheck {
  state: UpdateState;
  /** False on an unsigned macOS build, where updating means downloading by hand. */
  canInstallInPlace: boolean;
  update: UpdateInfo | null;
  progress: number;
  errorMessage: string | null;
  check: () => void;
  install: () => void;
  dismiss: () => void;
}

export function useUpdateCheck(autoCheckEnabled: boolean): UseUpdateCheck {
  const [state, setState] = useState<UpdateState>("idle");
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [progress, setProgress] = useState(0);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const pendingUpdate = useRef<Update | null>(null);
  const startupCheckFired = useRef(false);
  // Asked once per window. Until the answer lands, assume it is not allowed: offering
  // "update and restart" on a build that cannot do it safely is the wrong way to be
  // wrong — see `code_signature.rs`.
  const [canInstallInPlace, setCanInstallInPlace] = useState(false);

  useEffect(() => {
    canSelfInstall()
      .then(setCanInstallInPlace)
      .catch(() => setCanInstallInPlace(false));
  }, []);

  const runCheck = useCallback(async (isManual: boolean) => {
    setState("checking");
    setErrorMessage(null);
    try {
      // The plugin defaults to no timeout at all, so a stalled connection to GitHub
      // leaves this stuck on "checking" with the button disabled for the rest of the
      // session — and it runs automatically on every launch.
      const found = await check({ timeout: 15000 });
      if (!found) {
        setState("none");
        return;
      }
      pendingUpdate.current = found;
      setUpdate({
        version: found.version,
        body: found.body ?? null,
        date: found.date ?? null,
      });
      setState("available");
    } catch (err) {
      pendingUpdate.current = null;
      if (isManual) {
        setState("error");
        setErrorMessage(String(err));
      } else {
        setState("idle");
      }
    }
  }, []);

  useEffect(() => {
    if (!autoCheckEnabled || startupCheckFired.current) return;
    startupCheckFired.current = true;
    const id = window.setTimeout(() => runCheck(false), STARTUP_CHECK_DELAY_MS);
    return () => window.clearTimeout(id);
  }, [autoCheckEnabled, runCheck]);

  const install = useCallback(() => {
    if (!canInstallInPlace) {
      openReleasesPage();
      return;
    }
    const target = pendingUpdate.current;
    if (!target) return;
    setState("downloading");
    setProgress(0);
    let totalBytes = 0;
    let receivedBytes = 0;
    target
      .downloadAndInstall((event) => {
        if (event.event === "Started") {
          totalBytes = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          receivedBytes += event.data.chunkLength;
          setProgress(
            totalBytes > 0 ? Math.min(100, Math.round((receivedBytes / totalBytes) * 100)) : 0,
          );
        }
      })
      .then(() => {
        setState("installed");
        return relaunch();
      })
      .catch((err) => {
        setState("error");
        setErrorMessage(String(err));
        openReleasesPage();
      });
  }, [canInstallInPlace]);

  const dismiss = useCallback(() => setState("none"), []);

  return {
    state,
    canInstallInPlace,
    update,
    progress,
    errorMessage,
    check: () => runCheck(true),
    install,
    dismiss,
  };
}
