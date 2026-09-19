/**
 * Press-to-set hotkey field.
 *
 * Reads `KeyboardEvent.code` rather than `.key`: on macOS, Option+D reports `.key`
 * as "∂", and the physical-key names are exactly what the accelerator parser accepts.
 * The app's own global shortcuts are released while recording, otherwise pressing the
 * combination you want to assign would trigger its action instead of being captured.
 *
 * A captured key is then checked against the rest of the machine before it is trusted.
 * On a Wayland desktop a shortcut the desktop already owns is accepted, saved, listed —
 * and never delivered, because the compositor consumes it first. That silence was the
 * whole of the "I set a hotkey and nothing happens" report, so the answer is shown here,
 * next to the key, along with combinations this machine still has free.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../lib/api";
import type { HotkeyCheck } from "../lib/types";
import { useT } from "../lib/i18n";
import { formatAccelerator } from "../lib/format-accelerator";

interface Props {
  value: string | null;
  onChange: (accelerator: string | null) => void;
  /** Shown when nothing is bound. */
  placeholder?: string;
}

/** Keys that are safe to bind on their own — everything else needs a modifier. */
function isSelfSufficient(code: string): boolean {
  return /^F\d{1,2}$/.test(code);
}

function buildAccelerator(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Command");

  const code = e.code;
  // Modifier-only presses are the user still assembling the combination.
  if (/^(Control|Alt|Shift|Meta)(Left|Right)$/.test(code) || code === "Fn") {
    return null;
  }
  if (parts.length === 0 && !isSelfSufficient(code)) {
    return null;
  }

  parts.push(code);
  return parts.join("+");
}

export function HotkeyRecorder({ value, onChange, placeholder }: Props) {
  const [recording, setRecording] = useState(false);
  const [hint, setHint] = useState<string | null>(null);
  const [check, setCheck] = useState<HotkeyCheck | null>(null);
  const [suggestions, setSuggestions] = useState<string[] | null>(null);
  const t = useT();
  const buttonRef = useRef<HTMLButtonElement>(null);

  const stop = useCallback(() => {
    setRecording(false);
    api.resumeHotkeys().catch(() => undefined);
  }, []);

  const start = useCallback(() => {
    setHint(null);
    setSuggestions(null);
    setRecording(true);
    api.suspendHotkeys().catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!recording) return;

    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.code === "Escape") {
        setHint(null);
        stop();
        return;
      }
      if (e.code === "Backspace" || e.code === "Delete") {
        onChange(null);
        stop();
        return;
      }

      const accelerator = buildAccelerator(e);
      if (!accelerator) {
        // Give feedback for the one case that looks broken: a bare letter.
        if (!/^(Control|Alt|Shift|Meta)(Left|Right)$/.test(e.code)) {
          setHint(t.recorder.needModifier);
        }
        return;
      }
      onChange(accelerator);
      stop();
    };

    window.addEventListener("keydown", onKeyDown, true);
    // Losing focus mid-recording would otherwise leave hotkeys suspended.
    window.addEventListener("blur", stop);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("blur", stop);
    };
  }, [recording, onChange, stop]);

  // Safety net: never leave the app without its hotkeys because a tab was switched.
  useEffect(() => stop, [stop]);

  // Verdict on whatever the field holds now. Runs on mount too: a shortcut saved months
  // ago can be swallowed by a desktop shortcut added yesterday, and the user has no way
  // of learning that except by pressing the key and getting nothing.
  useEffect(() => {
    if (!value) {
      setCheck(null);
      return;
    }
    let live = true;
    api
      // The field's own key is passed as `current`, which is what stops a saved
      // shortcut from being reported as clashing with itself.
      .hotkeyCheck(value, value)
      .then((result) => {
        if (live) setCheck(result);
      })
      .catch(() => {
        if (live) setCheck(null);
      });
    return () => {
      live = false;
    };
  }, [value]);

  const problem = check && check.status !== "ok" ? check : null;
  // A refused registration on Windows or X11 says only that the grab failed, never by
  // whom — so an owner-less clash gets its own wording rather than a blank name.
  const problemText = problem
    ? (problem.status === "taken_by_desktop"
        ? problem.owner
          ? t.recorder.takenByDesktop
          : t.recorder.takenByOther
        : problem.status === "taken_by_app"
          ? t.recorder.takenByApp
          : t.recorder.unsupported
      ).replace("{owner}", problem.owner ?? "")
    : null;

  const offerSuggestions = () => {
    if (suggestions) {
      setSuggestions(null);
      return;
    }
    api
      .hotkeySuggestions()
      .then(setSuggestions)
      .catch(() => setSuggestions([]));
  };

  return (
    <div className="rec">
      <button
        ref={buttonRef}
        type="button"
        className={`rec__btn ${recording ? "rec__btn--live" : ""} ${
          problem ? "rec__btn--clash" : ""
        }`}
        onClick={() => (recording ? stop() : start())}
      >
        {recording ? (
          t.recorder.press
        ) : value ? (
          <kbd>{formatAccelerator(value)}</kbd>
        ) : (
          <span className="muted">{placeholder ?? t.recorder.clickToSet}</span>
        )}
      </button>

      {!recording && (
        <button type="button" className="btn-quiet" onClick={offerSuggestions}>
          {t.recorder.suggest}
        </button>
      )}

      {value && !recording && (
        <button type="button" className="btn-quiet btn-danger" onClick={() => onChange(null)}>
          {t.common.clear}
        </button>
      )}

      {recording && <span className="rec__note">{hint ?? t.recorder.hint}</span>}

      {!recording && problemText && <span className="rec__clash">{problemText}</span>}

      {!recording && suggestions && (
        <div className="rec__picks">
          {suggestions.length === 0 ? (
            <span className="rec__note">{t.recorder.suggestEmpty}</span>
          ) : (
            suggestions.map((accelerator) => (
              <button
                key={accelerator}
                type="button"
                className="rec__pick"
                onClick={() => {
                  onChange(accelerator);
                  setSuggestions(null);
                }}
              >
                <kbd>{formatAccelerator(accelerator)}</kbd>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}
