/** Subscribes to the backend event stream and exposes it as React state. */

import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { EVENTS } from "./api";
import type { Phase, StatusPayload } from "./types";

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
  error: string | null;
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
  // Tracks the phase across renders so the transcript can be cleared exactly once
  // per take, on the transition into `recording`.
  const phaseRef = useRef<Phase>("idle");

  useEffect(() => {
    const unlisteners: Array<Promise<() => void>> = [
      listen<StatusPayload>(EVENTS.status, ({ payload }) => {
        const startingNewTake =
          payload.phase === "recording" && phaseRef.current !== "recording";
        phaseRef.current = payload.phase;
        setState((prev) => ({
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
        }));
      }),

      listen<string>(EVENTS.partial, ({ payload }) => {
        setState((prev) => ({ ...prev, partial: payload }));
      }),

      listen<string>(EVENTS.transcript, ({ payload }) => {
        setState((prev) => ({
          ...prev,
          transcript: joinSegments(prev.transcript, payload),
          partial: "",
        }));
      }),

      listen<number>(EVENTS.level, ({ payload }) => {
        setState((prev) => ({
          ...prev,
          levels: [...prev.levels.slice(1), payload],
        }));
      }),

      listen<string>(EVENTS.error, ({ payload }) => {
        setState((prev) => ({ ...prev, error: payload }));
      }),
    ];

    return () => {
      unlisteners.forEach((p) => p.then((off) => off()).catch(() => undefined));
    };
  }, []);

  return state;
}

/**
 * Ticking `m:ss` for the take in progress. Only re-renders while something is
 * running, so an idle overlay costs nothing.
 */
export function useElapsed(startedAt: number | null): string | null {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (startedAt === null) return;
    setNow(Date.now());
    const id = window.setInterval(() => setNow(Date.now()), 500);
    return () => window.clearInterval(id);
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
