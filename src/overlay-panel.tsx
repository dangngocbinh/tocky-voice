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
import { formatAccelerator } from "./lib/format-accelerator";
import { useDictationEvents, useElapsed } from "./lib/use-dictation-events";
import { useT } from "./lib/i18n";
import { formatError } from "./lib/format-error";
import { CheckIcon, CopyIcon } from "./components/icons";
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
  // Exactly what is on screen, in the order it is read. Copying anything else would
  // be a button that quietly disagrees with the panel above it.
  const onScreen = [transcript, partial].filter(Boolean).join(" ");

  return (
    <div className="overlay">
      <div className="overlay__top">
        <span className={`overlay__dot overlay__dot--${phase}`} />
        <span className="overlay__phase">{t.phase[phase]}</span>
        {modeName && <span className="overlay__mode">{modeName}</span>}
        {elapsed && <span className="overlay__timer">{elapsed}</span>}
        {onScreen && <CopyButton text={onScreen} />}
      </div>

      <Waveform levels={levels} active={recording} size="sm" />

      {/* Outside the scrolling transcript on purpose: the text area follows the
          newest words to the bottom, which would carry a failure notice off the top
          of the panel exactly when it matters. */}
      {error && <div className="overlay__error">{formatError(error, t)}</div>}

      <div className="overlay__text" ref={textRef}>
        {onScreen ? (
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
          !error && <span className="overlay__hint">{t.overlay.speak}</span>
        )}
      </div>

      {recording && <Controls />}
    </div>
  );
}

/**
 * Takes the words off the panel and puts them on the clipboard.
 *
 * The panel is click-through and the text in it cannot be selected, so until this
 * existed the words on screen were only ever *about* to be delivered — and when a
 * take failed on its way to the target app, watching them disappear was the whole of
 * the experience. One button, present from the first word of every take.
 */
function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const t = useT();

  useEffect(() => {
    if (!copied) return;
    const id = window.setTimeout(() => setCopied(false), 1400);
    return () => window.clearTimeout(id);
  }, [copied]);

  const copy = () => {
    api
      .copyText(text)
      .then(() => setCopied(true))
      .catch(() => undefined);
  };

  return (
    <button
      className={`overlay__copy${copied ? " overlay__copy--done" : ""}`}
      onClick={copy}
      title={copied ? t.common.copied : t.common.copy}
    >
      {copied ? <CheckIcon /> : <CopyIcon />}
      <span>{copied ? t.common.copied : t.common.copy}</span>
    </button>
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

  const stop = formatAccelerator(settings?.hotkeys.toggle ?? null);

  return (
    <div className="overlay__foot">
      {/* Named after what the key does, not just what it is called. This panel is on
          screen at the exact moment someone is wondering how to end a take. */}
      <div className="overlay__keys">
        {stop && (
          <span className="overlay__key">
            <kbd>{stop}</kbd>
            <span>{t.overlay.stop}</span>
          </span>
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
