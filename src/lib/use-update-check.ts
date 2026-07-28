/** Checks GitHub for a newer release, and installs it where the platform allows. */

import { useCallback, useEffect, useRef, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { canSelfInstall } from "./update-policy";
import { openReleasesPage } from "./api";

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

  const runCheck = useCallback(async (isManual: boolean) => {
    setState("checking");
    setErrorMessage(null);
    try {
      const found = await check();
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
    if (!canSelfInstall()) {
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
  }, []);

  const dismiss = useCallback(() => setState("none"), []);

  return {
    state,
    update,
    progress,
    errorMessage,
    check: () => runCheck(true),
    install,
    dismiss,
  };
}
