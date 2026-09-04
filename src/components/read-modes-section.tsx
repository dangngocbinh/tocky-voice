/** Read-mode catalogue editor — the read-aloud equivalent of `modes-editor.tsx`,
 *  reusing `ModeList` / `PromptField` rather than duplicating them. Deliberately a
 *  separate list from dictation `modes`: a read mode's voice/speed and a dictation
 *  mode's output rule mean nothing to each other — see analysis §3.4. */

import { useState } from "react";
import { useT } from "../lib/i18n";
import type { AppSettings, ReadMode } from "../lib/types";
import { HotkeyRecorder } from "./hotkey-recorder";
import { ModeList } from "./mode-list";
import { PromptField } from "./prompt-field";
import { Switch } from "./providers-editor";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function ReadModesSection({ settings, onSettingsChange }: Props) {
  const [selectedId, setSelectedId] = useState(settings.active_read_mode_id);
  const t = useT();
  const selected =
    settings.read_modes.find((m) => m.id === selectedId) ?? settings.read_modes[0];

  const updateMode = (patch: Partial<ReadMode>) =>
    onSettingsChange({
      ...settings,
      read_modes: settings.read_modes.map((m) => (m.id === selected.id ? { ...m, ...patch } : m)),
    });

  const addMode = () => {
    const id = `read-mode-${Date.now()}`;
    onSettingsChange({
      ...settings,
      read_modes: [
        ...settings.read_modes,
        {
          id,
          name: t.modes.newModeName,
          hotkey: null,
          ai: true,
          prompt: t.modes.newModePrompt,
          llm_override: null,
        },
      ],
    });
    setSelectedId(id);
  };

  const removeMode = (id: string) => {
    if (settings.read_modes.length <= 1) return;
    const read_modes = settings.read_modes.filter((m) => m.id !== id);
    onSettingsChange({
      ...settings,
      read_modes,
      active_read_mode_id:
        settings.active_read_mode_id === id ? read_modes[0].id : settings.active_read_mode_id,
    });
    setSelectedId(read_modes[0].id);
  };

  return (
    <section className="section">
      <h2 className="section__title">{t.read.modesSection}</h2>
      <p className="row__hint">{t.read.modeListHint}</p>

      <div className="modes">
        <ModeList
          items={settings.read_modes.map((mode) => ({
            id: mode.id,
            name: mode.name,
            meta: [mode.ai ? t.read.modeAi : t.read.modeVerbatim, mode.hotkey ? t.modes.bound : null]
              .filter(Boolean)
              .join(" · "),
          }))}
          selectedId={selected.id}
          onSelect={setSelectedId}
          onAdd={addMode}
          addLabel={t.read.addMode}
        />

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
              <div className="row__label">{t.read.aiToggle}</div>
              <span className="row__hint">{t.read.aiToggleHint}</span>
            </div>
            <div className="row__control">
              <Switch checked={selected.ai} onChange={(ai) => updateMode({ ai })} />
            </div>
          </div>

          <PromptField
            label={t.modes.prompt}
            hint={t.read.promptHint}
            disabled={!selected.ai}
            value={selected.prompt}
            onChange={(prompt) => updateMode({ prompt })}
            rows={8}
          />

          <div className="row row--tight">
            <span className="row__hint">{settings.read_modes.length <= 1 ? t.modes.lastMode : ""}</span>
            <div className="row__control">
              <button
                className="btn-quiet btn-danger"
                onClick={() => removeMode(selected.id)}
                disabled={settings.read_modes.length <= 1}
              >
                {t.modes.deleteMode}
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
