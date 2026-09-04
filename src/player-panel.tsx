/**
 * The floating read-aloud control bar. Same "must not steal focus" contract as
 * `overlay-panel.tsx`, and for the same underlying reason: bumping focus off whatever
 * the user is reading breaks arrow-key scrolling there. Unlike the overlay, the
 * buttons here are real (not just a keyboard reminder) — clicking one is safe because
 * `read/mod.rs` hands focus back to the target app after every command.
 */

import { useEffect, useRef, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import * as api from "./lib/api";
import { useReadStatus } from "./lib/use-read-status";
import { useT } from "./lib/i18n";
import { formatError } from "./lib/format-error";
import type { AppSettings } from "./lib/types";

/** Position changes fire on every pixel of a drag; saving on each one would spam the
 *  settings file. Only the position after dragging stops is worth persisting. */
const SAVE_POSITION_DEBOUNCE_MS = 500;
/** The speed slider is dragged continuously; each stop costs a settings write and
 *  re-reads apply from the next chunk anyway, so there is nothing to gain from
 *  reacting to every intermediate value. */
const SPEED_DEBOUNCE_MS = 250;
/** Window for discarding the `onMoved` the backend's own opening `set_position`
 *  triggers — see the listener for why saving that one is actively harmful. */
const IGNORE_MOVES_AFTER_OPEN_MS = 1500;


function formatTime(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

export function PlayerPanel() {
  const status = useReadStatus();
  const t = useT();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [localSpeed, setLocalSpeed] = useState<number | null>(null);
  const saveTimer = useRef<number | undefined>(undefined);
  const speedTimer = useRef<number | undefined>(undefined);

  useEffect(() => {
    const load = () => api.getSettings().then(setSettings).catch(() => undefined);
    load();
    const off = listen(api.EVENTS.settingsChanged, load);
    return () => {
      off.then((fn) => fn()).catch(() => undefined);
    };
  }, []);

  useEffect(() => {
    const win = getCurrentWebviewWindow();
    // `onMoved` cannot tell a drag from the backend positioning the window as it opens,
    // and saving the latter writes the *default* spot back as though it were chosen —
    // which then outranks the default forever, so changing the default stops having
    // any effect for anyone who has run the app once. Ignore moves that land in the
    // opening moments; a real drag comes later, by hand.
    const openedAt = Date.now();
    const unlisten = win.onMoved(({ payload }) => {
      if (Date.now() - openedAt < IGNORE_MOVES_AFTER_OPEN_MS) return;
      window.clearTimeout(saveTimer.current);
      saveTimer.current = window.setTimeout(() => {
        api.savePlayerPosition(payload.x, payload.y).catch(() => undefined);
      }, SAVE_POSITION_DEBOUNCE_MS);
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => undefined);
    };
  }, []);

  if (!status) return null;

  const preparing = status.phase === "preparing";
  const paused = status.phase === "paused";
  const failed = status.phase === "failed";
  /** The capture came back empty but there is text on the clipboard: ask rather than
   *  dead-end. Its own layout, not a variation on the error row — this is a question
   *  with a button to press, and it needs the room the mode/speed row would take. */
  const clipboardPreview = failed ? status.clipboard_preview : null;
  const progressPct =
    status.total_ms > 0 ? Math.min(100, (status.played_ms / status.total_ms) * 100) : 0;
  const speed = localSpeed ?? status.speed;

  const changeSpeed = (next: number) => {
    setLocalSpeed(next);
    window.clearTimeout(speedTimer.current);
    speedTimer.current = window.setTimeout(() => {
      api
        .setReadSpeed(next)
        .catch(() => undefined)
        .finally(() => setLocalSpeed(null));
    }, SPEED_DEBOUNCE_MS);
  };

  if (clipboardPreview !== null) {
    return (
      // The whole card is a drag handle here — unlike the normal view below, this one
      // has no controls row underneath competing for mousedown, so there's no reason to
      // limit dragging to the top strip. "deep" makes every non-interactive descendant
      // (the preview box, the gap next to the action button) part of the drag region
      // too; the buttons still work because Tauri blocks dragging on clickable elements
      // that don't carry the attribute themselves.
      <div className="player" data-tauri-drag-region="deep">
        <div className="player__row">
          <p className="player__offer">{t.player.clipboardOffer}</p>
          <button
            className="player__btn"
            onClick={() => api.stopReading()}
            aria-label={t.player.close}
          >
            ✕
          </button>
        </div>
        {/* What the button is about to send to a speech vendor and say out loud, shown
            before it happens — the clipboard is where passwords and private messages
            live, and a blind "yes" there is the one expensive mistake this can make.
            One line, clipped: the backend already caps the length. */}
        <p className="player__preview" title={clipboardPreview}>
          {clipboardPreview}
        </p>
        <div className="player__offer-actions">
          <button className="player__action" onClick={() => api.readFromClipboard()}>
            {t.player.readClipboard}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="player">
      <div className="player__row" data-tauri-drag-region>
        <button
          className="player__btn"
          onClick={() => (paused ? api.resumeReading() : api.pauseReading())}
          disabled={preparing || failed}
          aria-label={paused ? t.player.resume : t.player.pause}
        >
          {paused ? "▶" : "⏸"}
        </button>

        <div className="player__progress">
          {failed ? (
            <span className="player__error">{formatError(status.error, t)}</span>
          ) : preparing ? (
            <>
              <div className="player__bar player__bar--indeterminate" />
              <span className="player__label">{t.player.preparing}</span>
            </>
          ) : (
            <>
              <div className="player__bar">
                <div className="player__bar-fill" style={{ width: `${progressPct}%` }} />
              </div>
              <span className="player__time">
                {formatTime(status.played_ms)} / {formatTime(status.total_ms)}
              </span>
            </>
          )}
        </div>

        <button className="player__btn" onClick={() => api.stopReading()} aria-label={t.player.close}>
          ✕
        </button>
      </div>

      {/* Always on screen rather than behind a disclosure toggle. Hiding two controls
          behind a ▾ meant a resize animation, a fiddly 26px hit target, and a second
          row that arrived cramped anyway — for a bar that is only up while reading, a
          fixed two-row layout with room to breathe is both simpler and easier to use. */}
      <div className="player__controls">
        <label className="player__field">
          <span className="player__field-label">{t.player.mode}</span>
          <select
            className="player__select"
            value={settings?.active_read_mode_id ?? ""}
            onChange={(e) => api.setReadMode(e.target.value).catch(() => undefined)}
          >
            {(settings?.read_modes ?? []).map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
        </label>

        <label className="player__field player__field--speed">
          <span className="player__field-label">
            {t.player.speedLabel} <b>{speed.toFixed(2)}x</b>
          </span>
          {/* Range comes from the status payload, which carries the active provider's
              own limits — Soniox accepts 0.7–1.3 while Vbee goes to 1.9, and a slider
              hardcoded to either one produces vendor rejections on the other. */}
          <input
            className="player__speed"
            type="range"
            min={status.speed_min}
            max={status.speed_max}
            step={0.05}
            value={speed}
            onChange={(e) => changeSpeed(Number(e.target.value))}
            title={t.player.speed}
          />
        </label>
      </div>
    </div>
  );
}
