/**
 * Step 4: prove the hotkey works somewhere that is not this window.
 *
 * This used to be a boxed waveform + Start button, which is the exact shape of the
 * microphone check one step earlier — same box, same bars moving, different purpose.
 * People landing here read it as "didn't I just do this?" The fix is to not look like
 * it: no waveform, no device picker (already chosen), just the keys, an instruction to
 * go use one in another app, and a checkmark once real text lands. The in-window button
 * stays as a quiet fallback for anyone whose hotkey registration isn't working yet, but
 * it shares the exact same status readout rather than a second stage box.
 */

import { useEffect, useRef, useState } from "react";
import * as api from "../lib/api";
import walkthroughVideo from "../assets/onboarding/walkthrough-en.mp4";
import { formatAccelerator, pushToTalkLabel } from "../lib/format-accelerator";
import { formatError } from "../lib/format-error";
import { useT } from "../lib/i18n";
import type { AppSettings } from "../lib/types";
import { useDictationEvents } from "../lib/use-dictation-events";

interface Props {
  settings: AppSettings;
}

type TryResult = "waiting" | "success" | "empty" | "error";

export function StepTryIt({ settings }: Props) {
  const { phase, transcript, error } = useDictationEvents();
  const t = useT();
  const [result, setResult] = useState<TryResult>("waiting");
  const prevPhase = useRef(phase);

  useEffect(() => {
    const prev = prevPhase.current;
    prevPhase.current = phase;
    if (phase === "recording" && prev !== "recording") {
      setResult("waiting");
    } else if (phase === "idle" && prev !== "idle" && prev !== "recording") {
      // A take ran the full pipeline and landed back on idle: read the verdict off
      // whatever the event stream settled on.
      setResult(error ? "error" : transcript.trim() ? "success" : "empty");
    }
  }, [phase, error, transcript]);

  // Outside macOS push-to-talk is bound to an accelerator rather than a held bare
  // modifier, and reading only the modifier case hid the hold key on every other
  // platform — on the one screen whose job is to teach it.
  const hold = pushToTalkLabel(settings.hotkeys);
  const toggle = formatAccelerator(settings.hotkeys.toggle);
  const cancel = formatAccelerator(settings.hotkeys.cancel);
  const recording = phase === "recording";

  return (
    <>
      <p className="onb__lede">{t.onboarding.tryBody}</p>

      {/* One real recording of the whole loop — hotkey, dictation, cursor placement —
          settles what a paragraph of instructions leaves people guessing about. */}
      <video className="onb__video" src={walkthroughVideo} controls preload="metadata" />
      <p className="onb__note">{t.onboarding.tryVideoCaption}</p>

      {/* Hold-to-talk leads: it is the faster of the two, and the one that is invisible
          unless someone says out loud that the key is held rather than pressed. */}
      <div className="onb__keys">
        {hold && (
          <div>
            <kbd>{hold}</kbd>
            <span>{t.onboarding.keyHold}</span>
          </div>
        )}
        <div>
          <kbd>{toggle}</kbd>
          <span>{t.onboarding.keyToggle}</span>
        </div>
        {cancel && (
          <div>
            <kbd>{cancel}</kbd>
            <span>{t.onboarding.keyCancel}</span>
          </div>
        )}
      </div>

      <p className="onb__note">{t.onboarding.tryNote}</p>

      <p className="onb__lede">
        {result === "waiting" && <span className="muted">{t.onboarding.tryWaiting}</span>}
        {result === "success" && <span className="onb__good">✓ {t.onboarding.trySuccess}</span>}
        {result === "empty" && <span className="muted">{t.onboarding.tryEmpty}</span>}
        {result === "error" && error && <span className="onb__bad">{formatError(error, t)}</span>}
      </p>

      <button
        className="btn-quiet"
        onClick={() => (recording ? api.stopRecording() : api.startRecording())}
      >
        {recording ? t.dictate.stop : t.onboarding.tryFallback}
      </button>
    </>
  );
}
