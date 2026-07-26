/**
 * Step 2: the speech provider and its key.
 *
 * This is the one step that cannot be skipped — without a key the app transcribes
 * nothing, and the failure at dictation time is a toast that vanishes. So the key is
 * saved and confirmed here, before anyone tries to use the app for real.
 */

import { useEffect, useState } from "react";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import type { AppSettings, SttProviderKind } from "../lib/types";
import { STT_PROVIDERS } from "../lib/types";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
  onReadyChange: (ready: boolean) => void;
}

export function StepSpeech({ settings, onSettingsChange, onReadyChange }: Props) {
  const [keys, setKeys] = useState<Record<string, boolean>>({});
  const [value, setValue] = useState("");
  const [saving, setSaving] = useState(false);
  const t = useT();

  const active = STT_PROVIDERS.find((p) => p.id === settings.stt.provider) ?? STT_PROVIDERS[0];
  const configured = !!keys[active.secret];

  const refresh = () => api.keyStatus().then(setKeys).catch(() => undefined);
  useEffect(() => {
    refresh();
  }, []);
  useEffect(() => onReadyChange(configured), [configured, onReadyChange]);

  const save = async () => {
    setSaving(true);
    try {
      await api.setApiKey(active.secret, value.trim());
      setValue("");
      await refresh();
    } finally {
      setSaving(false);
    }
  };

  return (
    <>
      <p className="onb__lede">{t.onboarding.speechBody}</p>

      <div className="onb__choices">
        {STT_PROVIDERS.map((p) => (
          <button
            key={p.id}
            className={`onb__choice ${settings.stt.provider === p.id ? "onb__choice--on" : ""}`}
            onClick={() =>
              onSettingsChange({
                ...settings,
                stt: { ...settings.stt, provider: p.id as SttProviderKind },
              })
            }
          >
            <span className="onb__choice-name">
              {p.label}
              {p.id === "soniox" && (
                <span className="chip chip--ok" style={{ marginLeft: 8 }}>
                  {t.onboarding.recommended}
                </span>
              )}
              {keys[p.secret] && <span className="onb__tick">✓</span>}
            </span>
            <span className="onb__choice-note">{p.note}</span>
          </button>
        ))}
      </div>

      {configured ? (
        <div className="onb__status onb__status--ok">
          <span className="onb__status-dot" />
          <span>{t.onboarding.keySaved.replace("{provider}", active.label)}</span>
        </div>
      ) : (
        <>
          <ol className="onb__list">
            <li>{t.onboarding.keyStep1.replace("{provider}", active.label)}</li>
            <li>{t.onboarding.keyStep2}</li>
          </ol>
          <div className="onb__actions">
            <button
              className="btn-quiet"
              onClick={() => api.openUrl(active.signupUrl).catch(() => undefined)}
            >
              {t.onboarding.getFreeKey} ↗
            </button>
            {active.freeCredit && <span className="chip chip--ok">{active.freeCredit}</span>}
          </div>
          <div className="keyline" style={{ marginTop: 12 }}>
            <input
              type="password"
              autoComplete="off"
              placeholder={t.providers.keyPlaceholder}
              value={value}
              onChange={(e) => setValue(e.target.value)}
            />
            <button className="btn-primary" onClick={save} disabled={saving || !value.trim()}>
              {t.common.save}
            </button>
          </div>
        </>
      )}
    </>
  );
}
