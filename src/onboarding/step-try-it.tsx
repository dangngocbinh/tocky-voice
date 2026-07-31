/**
 * Step 4: press a key, say a sentence, watch it land — right here.
 *
 * Sending people to another app to prove the hotkey works adds a second window to
 * juggle for something the event stream already tells us: recognized text arrives
 * over the same channel the floating overlay reads from, whether or not there was
 * anywhere to paste it. So this take is deliberately started from our own window,
 * and the overlay is told to stay hidden for it (`setOverlaySuppressed`) — otherwise
 * it pops up on top of this step for a take that has nowhere to paste into, which
 * reads as a second recording indicator fighting the one already on screen. A
 * `no_paste_target` error here is the expected outcome, not a failure: it means the
 * take started from this window, which is exactly what just happened.
 */

import { useEffect, useRef, useState } from "react";
import * as api from "../lib/api";
import { formatAccelerator, pushToTalkLabel } from "../lib/format-accelerator";
import { formatError } from "../lib/format-error";
import { resolveLanguage, useT } from "../lib/i18n";
import type { AppSettings } from "../lib/types";
import { useDictationEvents } from "../lib/use-dictation-events";

interface Props {
  settings: AppSettings;
}

type TryResult = "waiting" | "success" | "empty" | "error";

// Fetched from GitHub rather than bundled: a 1.4MB video baked into every install
// costs everyone the download, for a clip most people watch once. English-only
// narration for now, so it only makes sense to show — and only worth autoplaying —
// when the UI is already in English.
const WALKTHROUGH_VIDEO_URL =
  "https://raw.githubusercontent.com/dangngocbinh/tocky-voice/feature/onboarding-cancel-hint-and-demo-video/brand/videos/onboarding-walkthrough-en.mp4";
const WALKTHROUGH_PLAYBACK_RATE = 1.3;

export function StepTryIt({ settings }: Props) {
  const { phase, transcript, partial, error } = useDictationEvents();
  const t = useT();
  const [result, setResult] = useState<TryResult>("waiting");
  const prevPhase = useRef(phase);
  const videoRef = useRef<HTMLVideoElement>(null);
  const showVideo = resolveLanguage(settings.ui_language) === "en";

  useEffect(() => {
    if (videoRef.current) videoRef.current.playbackRate = WALKTHROUGH_PLAYBACK_RATE;
  }, [showVideo]);

  useEffect(() => {
    api.setOverlaySuppressed(true);
    return () => void api.setOverlaySuppressed(false);
  }, []);

  useEffect(() => {
    const prev = prevPhase.current;
    prevPhase.current = phase;
    if (phase === "recording" && prev !== "recording") {
      setResult("waiting");
    } else if (phase === "idle" && prev !== "idle" && prev !== "recording") {
      // A take ran the full pipeline and landed back on idle. `no_paste_target` is
      // the expected shape here — this take started from our own window on purpose
      // — so it reads as a normal landing, not a failure; anything else is real.
      const realError = error && error.kind !== "no_paste_target" ? error : null;
      setResult(realError ? "error" : transcript.trim() ? "success" : "empty");
    }
  }, [phase, error, transcript]);

  // Outside macOS push-to-talk is bound to an accelerator rather than a held bare
  // modifier, and reading only the modifier case hid the hold key on every other
  // platform — on the one screen whose job is to teach it.
  const hold = pushToTalkLabel(settings.hotkeys);
  const toggle = formatAccelerator(settings.hotkeys.toggle);
  const cancel = formatAccelerator(settings.hotkeys.cancel);
  const recording = phase === "recording";
  const text = [transcript, partial].filter(Boolean).join(" ");
  const displayError = error && error.kind !== "no_paste_target" ? error : null;

  return (
    <>
      <p className="onb__lede">{t.onboarding.tryBody}</p>

      {/* One real recording of the whole loop — hotkey, dictation, cursor placement —
          settles what a paragraph of instructions leaves people guessing about. English
          narration only, so it stays off for a Vietnamese UI rather than talk past the
          person reading it. */}
      {showVideo && (
        <>
          <video
            ref={videoRef}
            className="onb__video"
            src={WALKTHROUGH_VIDEO_URL}
            controls
            autoPlay
            preload="metadata"
          />
          <p className="onb__note">{t.onboarding.tryVideoCaption}</p>
        </>
      )}

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

      <div className="onb__stage">
        <div className="onb__stage-text">
          {text || <span className="muted">{t.onboarding.tryPreviewHint}</span>}
        </div>
        <button
          className={recording ? "btn-quiet btn-danger" : "btn-primary"}
          onClick={() => (recording ? api.stopRecording() : api.startRecording())}
        >
          {recording ? t.dictate.stop : t.onboarding.tryButton}
        </button>
      </div>

      <p className="onb__lede">
        {result === "waiting" && <span className="muted">{t.onboarding.tryWaiting}</span>}
        {result === "success" && <span className="onb__good">✓ {t.onboarding.trySuccess}</span>}
        {result === "empty" && <span className="muted">{t.onboarding.tryEmpty}</span>}
        {result === "error" && displayError && (
          <span className="onb__bad">{formatError(displayError, t)}</span>
        )}
      </p>
    </>
  );
}
