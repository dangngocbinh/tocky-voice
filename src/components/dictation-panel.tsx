/**
 * Home view: pick a microphone and mode, record, watch the text arrive.
 *
 * The mic picker lives here rather than buried in settings because switching inputs
 * (headset vs built-in) is something you do mid-session, not once during setup.
 */

import { useEffect, useState } from "react";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import { formatError } from "../lib/format-error";
import type { AppSettings, Phase } from "../lib/types";
import { useDictationEvents, useElapsed } from "../lib/use-dictation-events";
import { usePermissionStatus } from "../lib/use-permission-status";
import { Waveform } from "./waveform";

const BUSY: Phase[] = ["transcribing", "refining", "pasting"];

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function DictationPanel({ settings, onSettingsChange }: Props) {
  const { phase, transcript, partial, levels, startedAt, error } = useDictationEvents();
  const elapsed = useElapsed(startedAt);
  const [devices, setDevices] = useState<string[]>([]);
  const { accessibility } = usePermissionStatus();
  const t = useT();

  useEffect(() => {
    api.listInputDevices().then(setDevices).catch(() => setDevices([]));
  }, []);

  const recording = phase === "recording";
  const busy = BUSY.includes(phase);
  const text = [transcript, partial].filter(Boolean).join(" ");

  return (
    <>
      <h1 className="view__title">{t.dictate.title}</h1>
      <p className="view__lede">
{t.dictate.lede}
      </p>

      {!accessibility && (
        <div className="notice notice--warn">
          <div>
            <strong>{t.dictate.accessibilityTitle}</strong> {" "}{t.dictate.accessibilityBody}
          </div>
          <button onClick={() => api.openAccessibilitySettings()}>{t.common.openSettings}</button>
        </div>
      )}

      <div className="stage">
        <div className="stage__top">
          <span className="stage__phase">{t.phase[phase]}</span>
          {elapsed && <span className="stage__timer">{elapsed}</span>}
        </div>

        <Waveform levels={levels} active={recording} />

        <div className="stage__actions">
          <button
            className={recording ? "btn-danger" : "btn-primary"}
            onClick={() => api.toggleRecording()}
            disabled={busy}
          >
            {recording ? t.dictate.stop : busy ? t.dictate.working : t.dictate.start}
          </button>
          {recording && (
            <button className="btn-quiet" onClick={() => api.cancelRecording()}>
              {t.common.cancel}
            </button>
          )}
          {text && !recording && (
            <button className="btn-quiet" onClick={() => api.copyText(text)}>
              {t.dictate.copyText}
            </button>
          )}
        </div>

        <div className="transcript">
          {text ? (
            <>
              {transcript}
              {partial && (
                <span className="transcript__partial">
                  {transcript ? " " : ""}
                  {partial}
                </span>
              )}
            </>
          ) : (
<span className="transcript__empty">{t.dictate.empty}</span>
          )}
        </div>
      </div>

      {error && <div className="notice notice--error">{formatError(error, t)}</div>}

      <section className="section">
        <h2 className="section__title">{t.dictate.inputSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.dictate.microphone}</div>
<span className="row__hint">{t.dictate.microphoneHint}</span>
          </div>
          <div className="row__control">
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
              {devices.map((device) => (
                <option key={device} value={device}>
                  {device}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.dictate.mode}</div>
<span className="row__hint">{t.dictate.modeHint}</span>
          </div>
          <div className="row__control">
            <select
              value={settings.active_mode_id}
              onChange={(e) => {
                api.setActiveMode(e.target.value).catch(() => undefined);
                onSettingsChange({ ...settings, active_mode_id: e.target.value });
              }}
            >
              {settings.modes.map((mode) => (
                <option key={mode.id} value={mode.id}>
                  {mode.name}
                  {mode.ai_cleanup ? "" : ` — ${t.dictate.raw}`}
                </option>
              ))}
            </select>
          </div>
        </div>
      </section>
    </>
  );
}
