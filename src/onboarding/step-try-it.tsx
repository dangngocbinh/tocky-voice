/**
 * Step 4: one real dictation, then the sentence that matters.
 *
 * Reading that a hotkey exists is not the same as having pressed it once and watched
 * words appear. This step is deliberately a live take rather than a diagram: it also
 * triggers the microphone permission prompt here, in a screen that explains it, instead
 * of during the user's first real attempt in someone else's app.
 */

import { useEffect, useState } from "react";
import * as api from "../lib/api";
import { formatAccelerator, pushToTalkLabel } from "../lib/format-accelerator";
import { useT } from "../lib/i18n";
import type { AppSettings } from "../lib/types";
import { useDictationEvents } from "../lib/use-dictation-events";
import { Waveform } from "../components/waveform";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function StepTryIt({ settings, onSettingsChange }: Props) {
  const { phase, transcript, partial, levels } = useDictationEvents();
  const [devices, setDevices] = useState<string[]>([]);
  const t = useT();

  useEffect(() => {
    api.listInputDevices().then(setDevices).catch(() => setDevices([]));
  }, []);

  const recording = phase === "recording";
  const text = [transcript, partial].filter(Boolean).join(" ");
  // Outside macOS push-to-talk is bound to an accelerator rather than a held bare
  // modifier, and reading only the modifier case hid the hold key on every other
  // platform — on the one screen whose job is to teach it.
  const hold = pushToTalkLabel(settings.hotkeys);

  return (
    <>
      <p className="onb__lede">{t.onboarding.tryBody}</p>

      <div className="onb__row">
        <span className="onb__row-label">{t.dictate.microphone}</span>
        <select
          value={settings.audio.input_device ?? ""}
          onChange={(e) =>
            onSettingsChange({
              ...settings,
              audio: { ...settings.audio, input_device: e.target.value || null },
            })
          }
        >
          <option value="">{t.common.systemDefault}</option>
          {devices.map((d) => (
            <option key={d} value={d}>
              {d}
            </option>
          ))}
        </select>
      </div>

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
          <kbd>{formatAccelerator(settings.hotkeys.toggle)}</kbd>
          <span>{t.onboarding.keyToggle}</span>
        </div>
        {formatAccelerator(settings.hotkeys.cancel) && (
          <div>
            <kbd>{formatAccelerator(settings.hotkeys.cancel)}</kbd>
            <span>{t.onboarding.keyCancel}</span>
          </div>
        )}
      </div>

      <div className="onb__stage">
        <Waveform levels={levels} active={recording} />
        <div className="onb__stage-text">
          {text || <span className="muted">{t.onboarding.tryHint}</span>}
        </div>
        <button
          className={recording ? "btn-quiet btn-danger" : "btn-primary"}
          onClick={() => (recording ? api.stopRecording() : api.startRecording())}
        >
          {recording ? t.dictate.stop : t.onboarding.tryStart}
        </button>
      </div>

      {/* The payoff line. Everything above is setup; this is the actual product. */}
      <p className="onb__finale">{t.onboarding.tryFinale}</p>
    </>
  );
}
