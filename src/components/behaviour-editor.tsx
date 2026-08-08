/** Hotkeys, sound cues, history retention, and launch-at-login. */

import type { AppSettings } from "../lib/types";
import { useT } from "../lib/i18n";
import { HotkeyRecorder } from "./hotkey-recorder";
import { Switch } from "./providers-editor";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function BehaviourEditor({ settings, onSettingsChange }: Props) {
  const t = useT();

  const patchHotkeys = (patch: Partial<AppSettings["hotkeys"]>) =>
    onSettingsChange({ ...settings, hotkeys: { ...settings.hotkeys, ...patch } });

  const patchAudio = (patch: Partial<AppSettings["audio"]>) =>
    onSettingsChange({ ...settings, audio: { ...settings.audio, ...patch } });

  const patchHistory = (patch: Partial<AppSettings["history"]>) =>
    onSettingsChange({ ...settings, history: { ...settings.history, ...patch } });

  return (
    <>
      <h1 className="view__title">{t.behaviour.title}</h1>
      <p className="view__lede">
{t.behaviour.lede}
      </p>

      <section className="section">
        <h2 className="section__title">{t.behaviour.generalSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.uiLanguage}</div>
            <span className="row__hint">{t.behaviour.uiLanguageHint}</span>
          </div>
          <div className="row__control">
            <select
              value={settings.ui_language}
              onChange={(e) =>
                onSettingsChange({
                  ...settings,
                  ui_language: e.target.value as AppSettings["ui_language"],
                })
              }
            >
              <option value="system">{t.behaviour.followSystem}</option>
              <option value="en">English</option>
              <option value="vi">Tiếng Việt</option>
            </select>
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.onboarding.rerunTitle}</div>
            <span className="row__hint">{t.onboarding.rerunBody}</span>
          </div>
          <div className="row__control">
            <button
              onClick={() => onSettingsChange({ ...settings, onboarding_completed: false })}
            >
              {t.onboarding.rerun}
            </button>
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.update.autoCheckLabel}</div>
            <span className="row__hint">{t.update.autoCheckHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.auto_check_updates}
              onChange={(auto_check_updates) =>
                onSettingsChange({ ...settings, auto_check_updates })
              }
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.shortcutsSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.toggle}</div>
            <span className="row__hint">{t.behaviour.toggleHint}</span>
          </div>
          <div className="row__control">
            <HotkeyRecorder
              value={settings.hotkeys.toggle}
              onChange={(toggle) => patchHotkeys({ toggle })}
            />
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.cancel}</div>
            <span className="row__hint">{t.behaviour.cancelHint}</span>
          </div>
          <div className="row__control">
            <HotkeyRecorder
              value={settings.hotkeys.cancel}
              onChange={(cancel) => patchHotkeys({ cancel })}
            />
          </div>
        </div>

        <div className="row">
          <div className="row__label">{t.behaviour.nextMode}</div>
          <div className="row__control">
            <HotkeyRecorder
              value={settings.hotkeys.next_mode}
              onChange={(next_mode) => patchHotkeys({ next_mode })}
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.feedbackSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.sounds}</div>
<span className="row__hint">{t.behaviour.soundsHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.audio.feedback_sounds}
              onChange={(feedback_sounds) => patchAudio({ feedback_sounds })}
            />
          </div>
        </div>

        <div className="row">
          <div className="row__label">
            {t.behaviour.volume}
            <span className="row__hint mono">
              {Math.round(settings.audio.feedback_volume * 100)}%
            </span>
          </div>
          <div className="row__control">
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.audio.feedback_volume}
              onChange={(e) => patchAudio({ feedback_volume: Number(e.target.value) })}
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.historySection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.keepHistory}</div>
            <span className="row__hint">{t.behaviour.keepHistoryHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.history.enabled}
              onChange={(enabled) => patchHistory({ enabled })}
            />
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.keepAudio}</div>
            <span className="row__hint">{t.behaviour.keepAudioHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.history.keep_audio}
              onChange={(keep_audio) => patchHistory({ keep_audio })}
            />
          </div>
        </div>

        <div className="row">
          <div className="row__label">{t.behaviour.maxEntries}</div>
          <div className="row__control">
            <input
              type="number"
              min={10}
              max={5000}
              value={settings.history.max_entries}
              onChange={(e) => patchHistory({ max_entries: Number(e.target.value) })}
            />
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.retention}</div>
            <span className="row__hint">{t.behaviour.retentionHint}</span>
          </div>
          <div className="row__control">
            <input
              type="number"
              min={0}
              max={365}
              value={settings.history.audio_retention_days}
              onChange={(e) => patchHistory({ audio_retention_days: Number(e.target.value) })}
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.startupSection}</h2>
        <div className="row">
          <div className="row__label">{t.behaviour.autostart}</div>
          <div className="row__control">
            <Switch
              checked={settings.autostart}
              onChange={(autostart) => onSettingsChange({ ...settings, autostart })}
            />
          </div>
        </div>
      </section>
    </>
  );
}
