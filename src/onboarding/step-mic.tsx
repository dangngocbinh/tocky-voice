/**
 * Step 2: pick the microphone, and watch it move.
 *
 * This runs before the speech-provider step on purpose. Every later screen assumes
 * sound is reaching the app, and when it isn't, the failure surfaces as an empty
 * transcript — which reads as "the speech service is broken" and sends people off to
 * re-check an API key that was fine all along. The level meter turns that into
 * something the user can see in two seconds, before they have spent anything.
 *
 * Advancing is gated on the meter having actually moved. The wizard is skippable in
 * one click for anyone who knows better.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import { isMac } from "../lib/platform";
import type { AppSettings } from "../lib/types";
import { useDictationEvents } from "../lib/use-dictation-events";
import { Waveform } from "../components/waveform";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
  onReadyChange: (ready: boolean) => void;
}

/** Peak that counts as "the microphone is live". Mirrors `AUDIBLE_PEAK` in the backend. */
const AUDIBLE_PEAK = 0.01;

export function StepMic({ settings, onSettingsChange, onReadyChange }: Props) {
  const { levels } = useDictationEvents();
  const [devices, setDevices] = useState<string[]>([]);
  const [heard, setHeard] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const t = useT();

  const device = settings.audio.input_device;

  // One preview at a time, restarted whenever the picker changes. The backend closes
  // the previous stream itself, so switching device is a single call.
  useEffect(() => {
    let cancelled = false;
    setHeard(false);
    setError(null);
    api
      .startMicTest(device)
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
      api.stopMicTest().catch(() => undefined);
    };
  }, [device]);

  // Release the device on the way out — a driver that grants exclusive access would
  // otherwise refuse to open it again for the first real dictation.
  useEffect(() => () => void api.stopMicTest().catch(() => undefined), []);

  useEffect(() => {
    api.listInputDevices().then(setDevices).catch(() => setDevices([]));
  }, []);

  // Latching, not live: the check is "has this microphone ever produced sound", so a
  // pause between words must not take the green tick away again.
  const latest = levels[levels.length - 1] ?? 0;
  const heardRef = useRef(false);
  heardRef.current = heard;
  const markHeard = useCallback(() => {
    if (!heardRef.current) setHeard(true);
  }, []);
  useEffect(() => {
    if (latest >= AUDIBLE_PEAK) markHeard();
  }, [latest, markHeard]);

  useEffect(() => {
    onReadyChange(heard);
  }, [heard, onReadyChange]);

  return (
    <>
      <p className="onb__lede">{t.onboarding.micBody}</p>

      <div className="onb__row">
        <span className="onb__row-label">{t.dictate.microphone}</span>
        <select
          value={device ?? ""}
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

      <div className="onb__stage">
        <Waveform levels={levels} active={!error} />
        <div className="onb__stage-text">
          {error ? (
            <span className="onb__bad">
              {t.onboarding.micFailed} {error}
            </span>
          ) : heard ? (
            <span className="onb__good">{t.onboarding.micHeard}</span>
          ) : (
            <span className="muted">{t.onboarding.micWaiting}</span>
          )}
        </div>
      </div>

      {!heard && !error && (
        <p className="onb__note">
          {isMac ? t.onboarding.micSilentHintMac : t.onboarding.micSilentHintOther}
        </p>
      )}
    </>
  );
}
