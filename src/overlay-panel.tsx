/**
 * The floating readout shown while dictating from another app.
 *
 * Everything except the two buttons is click-through by design: this window must not
 * take keyboard focus, or the paste keystroke lands here instead of in the app the
 * user was typing into. The buttons are safe because the previously-frontmost app is
 * reactivated before the keystroke — see `focus.rs`.
 */

import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import * as api from "./lib/api";
import { formatAccelerator, formatModifier } from "./lib/format-accelerator";
import { useDictationEvents, useElapsed } from "./lib/use-dictation-events";
import { useT } from "./lib/i18n";
import { formatError } from "./lib/format-error";
import { Waveform } from "./components/waveform";
import type { AppSettings } from "./lib/types";

export function OverlayPanel() {
  const { phase, modeName, transcript, partial, levels, startedAt, error } =
    useDictationEvents();
  const elapsed = useElapsed(startedAt);
  const t = useT();
  const textRef = useRef<HTMLDivElement>(null);

  // Keep the newest words in view. The panel is a fixed height, so on a long dictation
  // the beginning scrolls away rather than the end being clipped — what you just said
  // is the part you need to see.
  useEffect(() => {
    const node = textRef.current;
    if (node) node.scrollTop = node.scrollHeight;
  }, [transcript, partial]);

  const recording = phase === "recording";

  return (
    <div className="overlay">
      <div className="overlay__top">
        <span className={`overlay__dot overlay__dot--${phase}`} />
        <span className="overlay__phase">{t.phase[phase]}</span>
        {modeName && <span className="overlay__mode">{modeName}</span>}
        {elapsed && <span className="overlay__timer">{elapsed}</span>}
      </div>

      <Waveform levels={levels} active={recording} size="sm" />

      <div className="overlay__text" ref={textRef}>
        {error ? (
          <span className="overlay__error">{formatError(error, t)}</span>
        ) : transcript || partial ? (
          <>
            {transcript}
            {partial && (
              <span className="overlay__partial">
                {transcript ? " " : ""}
                {partial}
              </span>
            )}
          </>
        ) : (
          <span className="overlay__hint">{t.overlay.speak}</span>
        )}
      </div>

      {recording && <Controls />}
    </div>
  );
}

/**
 * Both a keyboard reminder and real buttons. The overlay is click-through, so without
 * the reminder there is no visible way to end a take; without the buttons, anyone who
 * reaches for the mouse is stuck.
 */
function Controls() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const t = useT();

  useEffect(() => {
    const load = () => api.getSettings().then(setSettings).catch(() => undefined);
    load();
    const off = listen(api.EVENTS.settingsChanged, load);
    return () => {
      off.then((fn) => fn()).catch(() => undefined);
    };
  }, []);

  const ptt = settings?.hotkeys.push_to_talk;
  const release =
    ptt?.kind === "modifier"
      ? formatModifier(ptt.key)
      : ptt?.kind === "shortcut"
        ? formatAccelerator(ptt.accelerator)
        : null;
  const stop = formatAccelerator(settings?.hotkeys.toggle ?? null);

  return (
    <div className="overlay__foot">
      <div className="overlay__keys">
        {release && (
          <>
            <kbd>{release}</kbd>
            <span>{t.overlay.release}</span>
          </>
        )}
        {stop && (
          <>
            <kbd>{stop}</kbd>
            <span>{t.overlay.stop}</span>
          </>
        )}
      </div>

      <div className="overlay__buttons">
        <button className="btn-quiet" onClick={() => api.cancelRecording()}>
          {t.common.cancel}
        </button>
        <button className="btn-primary" onClick={() => api.stopRecording()}>
          {t.dictate.stop}
        </button>
      </div>
    </div>
  );
}
