/** Hotkeys, sound cues, history retention, and launch-at-login. */

import type { AppSettings, ModifierKey, PushToTalk } from "../lib/types";
import { useT } from "../lib/i18n";
import { isMac } from "../lib/platform";
import { HotkeyRecorder } from "./hotkey-recorder";
import { Switch } from "./providers-editor";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

/** Hold-to-talk is watched by a macOS-only event tap, so these are Mac key names. */
const MODIFIERS: Record<ModifierKey, string> = {
  right_option: "Right Option ⌥",
  left_option: "Left Option ⌥",
  right_command: "Right Command ⌘",
  fn: "Fn / Globe",
};

export function BehaviourEditor({ settings, onSettingsChange }: Props) {
  const t = useT();

  const patchHotkeys = (patch: Partial<AppSettings["hotkeys"]>) =>
    onSettingsChange({ ...settings, hotkeys: { ...settings.hotkeys, ...patch } });

  const patchAudio = (patch: Partial<AppSettings["audio"]>) =>
    onSettingsChange({ ...settings, audio: { ...settings.audio, ...patch } });

  const patchHistory = (patch: Partial<AppSettings["history"]>) =>
    onSettingsChange({ ...settings, history: { ...settings.history, ...patch } });

  const ptt = settings.hotkeys.push_to_talk;

  // Suggested binding when switching to "hold a combination". On a PC keyboard
  // Control+Alt is what AltGr sends, so a plain function key is both simpler to hold
  // and free of layout surprises; it matches the factory default on those platforms.
  const suggestedPttShortcut = isMac ? "Control+Alt+Space" : "F9";

  const setPttKind = (kind: PushToTalk["kind"]) =>
    patchHotkeys({
      push_to_talk:
        kind === "modifier"
          ? { kind: "modifier", key: "right_option" }
          : kind === "shortcut"
            ? { kind: "shortcut", accelerator: suggestedPttShortcut }
            : { kind: "disabled" },
    });

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
        <h2 className="section__title">{t.behaviour.pttSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.pttTrigger}</div>
            <span className="row__hint">{t.behaviour.pttTriggerHint}</span>
          </div>
          <div className="row__control">
            <select value={ptt.kind} onChange={(e) => setPttKind(e.target.value as PushToTalk["kind"])}>
              {/* Holding a bare modifier is watched by a macOS event tap. Offering it
                  elsewhere means the setting reads as configured and then silently does
                  nothing when the key is pressed — the worst kind of broken. */}
              {isMac && <option value="modifier">{t.behaviour.pttModifier}</option>}
              <option value="shortcut">{t.behaviour.pttShortcut}</option>
              <option value="disabled">{t.behaviour.pttDisabled}</option>
            </select>
          </div>
        </div>

        {ptt.kind === "modifier" && !isMac && (
          <div className="row">
            <div>
              <div className="row__label">{t.behaviour.pttMacOnly}</div>
              <span className="row__hint">{t.behaviour.pttMacOnlyHint}</span>
            </div>
            <div className="row__control">
              <button onClick={() => setPttKind("shortcut")}>{t.behaviour.pttUseShortcut}</button>
            </div>
          </div>
        )}

        {ptt.kind === "modifier" && isMac && (
          <div className="row">
            <div>
              <div className="row__label">{t.behaviour.pttKey}</div>
<span className="row__hint">{t.behaviour.pttKeyHint}</span>
            </div>
            <div className="row__control">
              <select
                value={ptt.key}
                onChange={(e) =>
                  patchHotkeys({
                    push_to_talk: { kind: "modifier", key: e.target.value as ModifierKey },
                  })
                }
              >
                {Object.entries(MODIFIERS).map(([key, label]) => (
                  <option key={key} value={key}>
                    {label}
                  </option>
                ))}
              </select>
            </div>
          </div>
        )}

        {ptt.kind === "shortcut" && (
          <div className="row">
            <div className="row__label">{t.behaviour.pttCombo}</div>
            <div className="row__control">
              <HotkeyRecorder
                value={ptt.accelerator}
                onChange={(accelerator) =>
                  patchHotkeys({
                    push_to_talk: accelerator
                      ? { kind: "shortcut", accelerator }
                      : { kind: "disabled" },
                  })
                }
              />
            </div>
          </div>
        )}
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
