/** Subscribes to the backend event stream and exposes it as React state. */

import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { EVENTS } from "./api";
import type { ErrorPayload, Phase, StatusPayload } from "./types";

/** How many level samples the waveform keeps on screen. */
const WAVEFORM_BARS = 48;

export interface DictationState {
  phase: Phase;
  modeName: string;
  modeId: string;
  /** Committed transcript text for the current take. */
  transcript: string;
  /** Interim hypothesis, replaced as the recognizer refines it. */
  partial: string;
  /** Rolling mic levels, oldest first. */
  levels: number[];
  /** `Date.now()` when the current take started; null when idle. */
  startedAt: number | null;
  error: ErrorPayload | null;
}

const INITIAL: DictationState = {
  phase: "idle",
  modeName: "",
  modeId: "",
  transcript: "",
  partial: "",
  levels: new Array(WAVEFORM_BARS).fill(0),
  startedAt: null,
  error: null,
};

export function useDictationEvents(): DictationState {
  const [state, setState] = useState<DictationState>(INITIAL);
  // The stream is folded into this ref first and only copied into React state while
  // the page is visible. A hidden WKWebView skips its rendering updates, and WebKit
  // keeps what every DOM change made in the meantime instead of releasing it — so a
  // settings window closed to the menu bar, re-rendering the level meter ten times a
  // second on every take, reached 3.6 GB over two days of normal use (measured: 325 MB
  // to 930 MB in 200 simulated takes, flat at ~55 MB with this gate). Hidden, this
  // costs a property write per event and nothing else.
  const latest = useRef<DictationState>(INITIAL);

  useEffect(() => {
    const apply = (next: (prev: DictationState) => DictationState) => {
      latest.current = next(latest.current);
      if (!document.hidden) setState(latest.current);
    };
    // Coming back into view: one render catches up with everything missed.
    const onVisibility = () => {
      if (!document.hidden) setState(latest.current);
    };
    document.addEventListener("visibilitychange", onVisibility);

    const unlisteners: Array<Promise<() => void>> = [
      listen<StatusPayload>(EVENTS.status, ({ payload }) => {
        apply((prev) => {
          // Cleared exactly once per take, on the transition into `recording`.
          const startingNewTake =
            payload.phase === "recording" && prev.phase !== "recording";
          return {
            ...prev,
            phase: payload.phase,
            modeId: payload.mode_id,
            modeName: payload.mode_name,
            transcript: startingNewTake ? "" : prev.transcript,
            partial: startingNewTake ? "" : prev.partial,
            error: startingNewTake ? null : prev.error,
            startedAt: startingNewTake
              ? Date.now()
              : payload.phase === "idle"
                ? null
                : prev.startedAt,
          };
        });
      }),

      listen<string>(EVENTS.partial, ({ payload }) => {
        apply((prev) => ({ ...prev, partial: payload }));
      }),

      listen<string>(EVENTS.transcript, ({ payload }) => {
        apply((prev) => ({
          ...prev,
          transcript: joinSegments(prev.transcript, payload),
          partial: "",
        }));
      }),

      listen<number>(EVENTS.level, ({ payload }) => {
        apply((prev) => ({
          ...prev,
          levels: [...prev.levels.slice(1), payload],
        }));
      }),

      listen<ErrorPayload>(EVENTS.error, ({ payload }) => {
        apply((prev) => ({ ...prev, error: payload }));
      }),
    ];

    return () => {
      document.removeEventListener("visibilitychange", onVisibility);
      unlisteners.forEach((p) => p.then((off) => off()).catch(() => undefined));
    };
  }, []);

  return state;
}

/**
 * Ticking `m:ss` for the take in progress. Only re-renders while something is
 * running and the page is on screen, so an idle or hidden window costs nothing.
 */
export function useElapsed(startedAt: number | null): string | null {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (startedAt === null) return;
    // Same reason as above: a hidden page must not keep re-rendering, and a window
    // closed mid-take would otherwise tick until it is next opened — possibly days.
    const tick = () => {
      if (!document.hidden) setNow(Date.now());
    };
    tick();
    const id = window.setInterval(tick, 500);
    document.addEventListener("visibilitychange", tick);
    return () => {
      window.clearInterval(id);
      document.removeEventListener("visibilitychange", tick);
    };
  }, [startedAt]);

  if (startedAt === null) return null;
  const total = Math.max(0, Math.floor((now - startedAt) / 1000));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

/** Mirrors the backend's joining rule so the preview matches what gets pasted. */
function joinSegments(existing: string, segment: string): string {
  const next = segment.trim();
  if (!next) return existing;
  const attachesToPreviousWord = ",.!?;:".includes(next[0]);
  const needsSpace =
    existing.length > 0 && !existing.endsWith(" ") && !attachesToPreviousWord;
  return existing + (needsSpace ? " " : "") + next;
}
