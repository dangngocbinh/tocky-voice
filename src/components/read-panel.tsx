/**
 * The "Đọc" tab: off by default (see ui-spec §2.1), showing only an explainer and one
 * button until turned on. Once on, hides nothing — voice, read modes and the flow-C
 * voice-command shortcut are all live edits, same as every other settings tab.
 */

import { useT } from "../lib/i18n";
import { formatAccelerator } from "../lib/format-accelerator";
import type { AppSettings } from "../lib/types";
import { SpeakerIcon } from "./icons";
import { HotkeyRecorder } from "./hotkey-recorder";
import { ReadVoiceSection } from "./read-voice-section";
import { ReadModesSection } from "./read-modes-section";
import { Switch } from "./providers-editor";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function ReadPanel({ settings, onSettingsChange }: Props) {
  const t = useT();

  const setEnabled = (enabled: boolean) =>
    onSettingsChange({ ...settings, tts: { ...settings.tts, enabled } });

  if (!settings.tts.enabled) {
    return (
      <div className="read-off">
        <SpeakerIcon className="read-off__icon" />
        <h1 className="view__title">{t.read.title}</h1>
        <p className="view__lede">{t.read.offLede}</p>
        <p className="muted">{t.read.offKeyNote}</p>
        <button className="btn-primary" onClick={() => setEnabled(true)}>
          {t.read.enable}
        </button>
      </div>
    );
  }

  const readHotkey = formatAccelerator(settings.hotkeys.read);
  const voiceCommandHotkey = formatAccelerator(settings.hotkeys.read_with_voice);

  return (
    <>
      <div className="row row--tight">
        <h1 className="view__title" style={{ marginBottom: 0 }}>
          {t.read.title}
        </h1>
        <div className="row__control">
          <span className="muted">{t.read.enabledLabel}</span>
          <Switch checked={settings.tts.enabled} onChange={setEnabled} />
        </div>
      </div>
      <p className="view__lede">
        {readHotkey ? t.read.onLede.replace("{hotkey}", readHotkey) : t.read.offLede}
      </p>

      <div className="row">
        <div className="row__label">{t.read.readHotkey}</div>
        <div className="row__control">
          <HotkeyRecorder
            value={settings.hotkeys.read}
            onChange={(read) => onSettingsChange({ ...settings, hotkeys: { ...settings.hotkeys, read } })}
          />
        </div>
      </div>

      <ReadVoiceSection settings={settings} onSettingsChange={onSettingsChange} />
      <ReadModesSection settings={settings} onSettingsChange={onSettingsChange} />

      <section className="section">
        <h2 className="section__title">{t.read.voiceCommandSection}</h2>
        <p className="row__hint">
          {t.read.voiceCommandBody.replace("{hotkey}", voiceCommandHotkey ?? "—")}
        </p>
        <div className="row">
          <div className="row__label">{t.read.voiceCommandHotkey}</div>
          <div className="row__control">
            <HotkeyRecorder
              value={settings.hotkeys.read_with_voice}
              onChange={(read_with_voice) =>
                onSettingsChange({ ...settings, hotkeys: { ...settings.hotkeys, read_with_voice } })
              }
            />
          </div>
        </div>
      </section>

      <p className="row__hint" style={{ marginTop: 20 }}>
        {t.read.privacy}
      </p>
    </>
  );
}
