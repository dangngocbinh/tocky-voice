/** Subscribes to the read-aloud status stream — the player's only data source. */

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import * as api from "./api";
import { EVENTS } from "./api";
import type { ReadStatusPayload } from "./types";

/** `null` only when nothing is being read. */
export function useReadStatus(): ReadStatusPayload | null {
  const [status, setStatus] = useState<ReadStatusPayload | null>(null);

  useEffect(() => {
    // Pull the current state as well as subscribing. Subscribing alone loses the very
    // first read of the session: the backend emits `preparing` in order to open this
    // window, and this webview is still booting at that moment, so the first event it
    // would otherwise see is `speaking` — i.e. the player appears only once audio is
    // already playing, which reads as "did my keypress even register?".
    let live = true;
    api
      .getReadStatus()
      .then((initial) => {
        // Never clobber a real event that landed while this call was in flight.
        if (live && initial) setStatus((prev) => prev ?? initial);
      })
      .catch(() => undefined);

    const off = listen<ReadStatusPayload>(EVENTS.readStatus, ({ payload }) => {
      setStatus(payload);
    });
    return () => {
      live = false;
      off.then((fn) => fn()).catch(() => undefined);
    };
  }, []);

  return status;
}
