/**
 * Live Accessibility permission state.
 *
 * Checking once on mount is not enough: granting the permission happens in System
 * Settings, in another app, while this window sits untouched in the background. A
 * one-shot check leaves the warning on screen after the user has already done what it
 * asked — which reads as the app being broken. Re-checking whenever the window comes
 * back into focus catches exactly the moment they return from granting it.
 */

import { useCallback, useEffect, useState } from "react";
import * as api from "./api";

/** Backstop for the case where focus never changes — settings granted via a script,
 *  or the window left visible on a second monitor the whole time. */
const POLL_MS = 3000;

export function usePermissionStatus() {
  // Assume granted until told otherwise, so a first paint never flashes a warning at
  // someone who has already set the app up.
  const [accessibility, setAccessibility] = useState(true);

  const refresh = useCallback(
    () =>
      api
        .permissionStatus()
        .then((s) => setAccessibility(s.accessibility))
        .catch(() => undefined),
    [],
  );

  useEffect(() => {
    refresh();
    const id = window.setInterval(refresh, POLL_MS);
    window.addEventListener("focus", refresh);
    return () => {
      window.clearInterval(id);
      window.removeEventListener("focus", refresh);
    };
  }, [refresh]);

  return { accessibility, refresh };
}
