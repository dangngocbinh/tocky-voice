/**
 * Modes are the core customisation: a prompt plus a delivery rule, so the same voice
 * input can become clean prose, a coding prompt, or a formal email.
 */

import { useState } from "react";
import type { AppSettings, Mode } from "../lib/types";
import { useT } from "../lib/i18n";
import { HotkeyRecorder } from "./hotkey-recorder";
import { Switch } from "./providers-editor";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function ModesEditor({ settings, onSettingsChange }: Props) {
  const [selectedId, setSelectedId] = useState(settings.active_mode_id);
  const t = useT();
  const selected = settings.modes.find((m) => m.id === selectedId) ?? settings.modes[0];

  const updateMode = (patch: Partial<Mode>) =>
    onSettingsChange({
      ...settings,
      modes: settings.modes.map((m) => (m.id === selected.id ? { ...m, ...patch } : m)),
    });

  const addMode = () => {
    const id = `mode-${Date.now()}`;
    onSettingsChange({
      ...settings,
      modes: [
        ...settings.modes,
        {
          id,
          name: t.modes.newModeName,
          hotkey: null,
          ai_cleanup: true,
          prompt: t.modes.newModePrompt,
          llm_override: null,
          output: "paste",
        },
      ],
    });
    setSelectedId(id);
  };

  const removeMode = (id: string) => {
    // Keep at least one: dictation has nowhere to go without a mode.
    if (settings.modes.length <= 1) return;
    const modes = settings.modes.filter((m) => m.id !== id);
    onSettingsChange({
      ...settings,
      modes,
      active_mode_id: settings.active_mode_id === id ? modes[0].id : settings.active_mode_id,
    });
    setSelectedId(modes[0].id);
  };

  return (
    <>
      <h1 className="view__title">{t.modes.title}</h1>
      <p className="view__lede">
{t.modes.lede}
      </p>

      <div className="modes">
        <aside className="modes__list">
          {settings.modes.map((mode) => (
            <button
              key={mode.id}
              className={`modes__item ${mode.id === selected.id ? "modes__item--on" : ""}`}
              onClick={() => setSelectedId(mode.id)}
            >
              <span className="modes__name">{mode.name}</span>
              <span className="modes__meta">
                {mode.ai_cleanup ? "AI" : "RAW"}
                {mode.hotkey ? ` · ${t.modes.bound}` : ""}
                {settings.active_mode_id === mode.id ? ` · ${t.modes.active}` : ""}
              </span>
            </button>
          ))}
          <button className="btn-quiet modes__add" onClick={addMode}>
            {t.modes.add}
          </button>
        </aside>

        <div>
          <div className="row">
            <div className="row__label">{t.modes.name}</div>
            <div className="row__control">
              <input value={selected.name} onChange={(e) => updateMode({ name: e.target.value })} />
            </div>
          </div>

          <div className="row">
            <div>
              <div className="row__label">{t.modes.hotkey}</div>
              <span className="row__hint">{t.modes.hotkeyHint}</span>
            </div>
            <div className="row__control">
              <HotkeyRecorder
                value={selected.hotkey}
                onChange={(hotkey) => updateMode({ hotkey })}
                placeholder={t.modes.notSet}
              />
            </div>
          </div>

          <div className="row">
            <div>
              <div className="row__label">{t.modes.aiCleanup}</div>
<span className="row__hint">{t.modes.aiCleanupHint}</span>
            </div>
            <div className="row__control">
              <Switch
                checked={selected.ai_cleanup}
                onChange={(ai_cleanup) => updateMode({ ai_cleanup })}
              />
            </div>
          </div>

          <div className="row">
            <div className="row__label">{t.modes.output}</div>
            <div className="row__control">
              <select
                value={selected.output}
                onChange={(e) => updateMode({ output: e.target.value as Mode["output"] })}
              >
                <option value="paste">{t.modes.outputPaste}</option>
                <option value="copy_only">{t.modes.outputCopy}</option>
              </select>
            </div>
          </div>

          <div className="row row--stack">
            <div className="row__label">
              {t.modes.prompt}
              <span className="row__hint">{t.modes.promptHint}</span>
            </div>
            <textarea
              rows={13}
              disabled={!selected.ai_cleanup}
              value={selected.prompt}
              onChange={(e) => updateMode({ prompt: e.target.value })}
            />
          </div>

          <div className="row row--tight">
            <span className="row__hint">
              {settings.modes.length <= 1 ? t.modes.lastMode : ""}
            </span>
            <div className="row__control">
              <button
                className="btn-quiet btn-danger"
                onClick={() => removeMode(selected.id)}
                disabled={settings.modes.length <= 1}
              >
                {t.modes.deleteMode}
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
