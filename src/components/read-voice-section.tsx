/** Provider / model / voice / speed picker for the "Đọc" tab, plus the "Nghe thử"
 *  preview button — the one control on this page that actually matters, since picking
 *  a voice you can't hear is just guessing.
 *
 *  Model and voice are both plain dropdowns fed by the provider's own catalogue: a
 *  list is a list, and a free-text box next to it only invites typing an id that does
 *  not exist. The speed slider's bounds come from the selected model for the same
 *  reason — Soniox accepts 0.7–1.3, so a 0.5–2.0 slider just manufactures rejections.
 */

import { useEffect, useState } from "react";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import { TTS_PROVIDERS } from "../lib/types";
import type { AppSettings, TtsModel, TtsVoice } from "../lib/types";
import { KeyField } from "./providers-editor";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function ReadVoiceSection({ settings, onSettingsChange }: Props) {
  const t = useT();
  const [models, setModels] = useState<TtsModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [catalogError, setCatalogError] = useState<string | null>(null);
  const [previewing, setPreviewing] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [keys, setKeys] = useState<Record<string, boolean>>({});
  const provider = TTS_PROVIDERS.find((p) => p.id === settings.tts.provider) ?? TTS_PROVIDERS[0];

  const refreshKeys = () => api.keyStatus().then(setKeys).catch(() => undefined);

  useEffect(() => {
    refreshKeys();
  }, []);

  // Refetching is keyed on provider and app id only. Including the whole `tts` object
  // would refire on every keystroke of the speed slider — each one a paid-ish API call
  // and a dropdown that flickers empty mid-edit.
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setCatalogError(null);
    setPreviewError(null);
    api
      .listTtsVoices(settings.tts)
      .then((list) => {
        if (cancelled) return;
        setModels(list);
        // Land on a valid model/voice for this provider rather than keeping ids from
        // the previous one, which is what produced `Invalid voice 'tts-rt-v1'`.
        const model = list.find((m) => m.id === settings.tts.model) ?? list[0];
        if (!model) return;
        const voiceOk = model.voices.some((v) => v.id === settings.tts.voice);
        if (model.id !== settings.tts.model || !voiceOk) {
          onSettingsChange({
            ...settings,
            tts: {
              ...settings.tts,
              model: model.id,
              voice: voiceOk ? settings.tts.voice : (model.voices[0]?.id ?? ""),
              speed: clamp(settings.tts.speed, model),
            },
          });
        }
      })
      .catch((e) => {
        if (cancelled) return;
        setModels([]);
        setCatalogError(String(e));
      })
      .finally(() => !cancelled && setLoading(false));
    return () => {
      cancelled = true;
    };
  }, [settings.tts.provider, settings.tts.vbee_app_id]); // eslint-disable-line react-hooks/exhaustive-deps

  const selectedModel = models.find((m) => m.id === settings.tts.model) ?? models[0];
  const voices = selectedModel?.voices ?? [];
  // Grouped by language so a long list (Vbee returns hundreds) is navigable at all.
  // Voices with no language stay in one unlabelled group at the end.
  const voiceGroups = groupByLanguage(voices);

  const preview = async () => {
    setPreviewing(true);
    setPreviewError(null);
    try {
      await api.previewVoice(settings.tts);
    } catch (e) {
      // A silent failure here is worse than an ugly one: this button exists so a
      // wrong provider/voice/key shows up as "why is nothing happening" the first
      // time, not two steps later mid-article.
      setPreviewError(String(e));
    } finally {
      window.setTimeout(() => setPreviewing(false), 1500);
    }
  };

  return (
    <section className="section">
      <h2 className="section__title">{t.read.voiceSection}</h2>

      <div className="row">
        <div className="row__label">{t.read.provider}</div>
        <div className="row__control">
          <select
            value={settings.tts.provider}
            onChange={(e) => {
              // Model/voice ids are provider-specific; the catalogue effect above
              // picks valid ones once the new provider's list arrives.
              const next = TTS_PROVIDERS.find((p) => p.id === e.target.value) ?? provider;
              onSettingsChange({
                ...settings,
                tts: { ...settings.tts, provider: next.id, voice: "", model: next.defaultModel },
              });
            }}
          >
            {TTS_PROVIDERS.map((p) => (
              <option key={p.id} value={p.id}>
                {p.label}
              </option>
            ))}
          </select>
          <span className={`chip ${keys[provider.secret] ? "chip--ok" : "chip--todo"}`}>
            {keys[provider.secret] ? t.providers.keySet : t.providers.noKey}
          </span>
        </div>
      </div>

      <div className="row row--stack">
        <div className="row__label">
          {provider.needsAppId ? t.read.credentials : t.providers.apiKey}
          <span className="row__hint">
            {provider.needsAppId
              ? t.read.appIdHint
              : provider.needsNewKey
                ? t.providers.apiKeyHint
                : t.read.providerReuseNote}{" "}
            <a
              href="#"
              onClick={(e) => {
                e.preventDefault();
                api.openUrl(provider.signupUrl).catch(() => undefined);
              }}
            >
              {t.providers.getKeyFrom} {provider.label} ↗
            </a>
          </span>
        </div>
        {/* Vbee authenticates with two values. Splitting them across a plain settings
            field (auto-saved on a debounce) and a KeyField (saved by its own button)
            meant filling in one and losing the other — so both are entered here and
            committed together by a single Save. */}
        {provider.needsAppId ? (
          <VbeeCredentials
            appId={settings.tts.vbee_app_id}
            configured={keys[provider.secret]}
            onSave={(appId, token) => {
              onSettingsChange({ ...settings, tts: { ...settings.tts, vbee_app_id: appId } });
              return api.setApiKey(provider.secret, token).then(refreshKeys);
            }}
          />
        ) : (
          <KeyField
            account={provider.secret}
            configured={keys[provider.secret]}
            onSaved={refreshKeys}
          />
        )}
      </div>

      {/* Hidden entirely for a provider with only one model (Vbee) — a dropdown with a
          single entry is a label pretending to be a control. */}
      {models.length > 1 && (
        <div className="row">
          <div className="row__label">{t.read.model}</div>
          <div className="row__control">
            <select
              value={selectedModel?.id ?? ""}
              onChange={(e) => {
                const model = models.find((m) => m.id === e.target.value);
                if (!model) return;
                onSettingsChange({
                  ...settings,
                  tts: {
                    ...settings.tts,
                    model: model.id,
                    // Voice lists differ per model; keep the current one only if the
                    // new model actually has it.
                    voice: model.voices.some((v) => v.id === settings.tts.voice)
                      ? settings.tts.voice
                      : (model.voices[0]?.id ?? ""),
                    speed: clamp(settings.tts.speed, model),
                  },
                });
              }}
            >
              {models.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.label}
                </option>
              ))}
            </select>
          </div>
        </div>
      )}

      <div className="row">
        <div className="row__label">{t.read.voice}</div>
        <div className="row__control">
          {/* One control, not a hand-rolled filter box beside it: a native select is
              already searchable — macOS jumps to entries as you type into an open one —
              and Vbee's several hundred voices are grouped by language so the right
              section is one scroll away. */}
          <select
            value={settings.tts.voice}
            disabled={loading || !voices.length}
            onChange={(e) =>
              onSettingsChange({ ...settings, tts: { ...settings.tts, voice: e.target.value } })
            }
          >
            {loading && <option value="">{t.read.voiceLoading}</option>}
            {!loading && !voices.length && <option value="">{t.read.voiceNone}</option>}
            {voiceGroups.map(([language, group]) =>
              language ? (
                <optgroup key={language} label={language}>
                  {group.map((v) => (
                    <option key={v.id} value={v.id}>
                      {v.label}
                    </option>
                  ))}
                </optgroup>
              ) : (
                group.map((v) => (
                  <option key={v.id} value={v.id}>
                    {v.label}
                  </option>
                ))
              ),
            )}
          </select>
          <button onClick={preview} disabled={previewing || loading}>
            {previewing ? t.read.previewing : t.read.preview}
          </button>
        </div>
      </div>

      {(catalogError || previewError) && (
        <div className="row row--tight">
          <span className="row__error">{catalogError ?? previewError}</span>
        </div>
      )}

      {selectedModel?.supports_speed && (
        <div className="row">
          <div className="row__label">{t.read.speed}</div>
          <div className="row__control">
            <input
              type="range"
              min={selectedModel.speed_min}
              max={selectedModel.speed_max}
              step={0.05}
              value={clamp(settings.tts.speed, selectedModel)}
              onChange={(e) =>
                onSettingsChange({
                  ...settings,
                  tts: { ...settings.tts, speed: Number(e.target.value) },
                })
              }
            />
            <span className="muted">{settings.tts.speed.toFixed(2)}x</span>
          </div>
        </div>
      )}
      {selectedModel && !selectedModel.supports_speed && (
        <div className="row">
          <div className="row__label">{t.read.speed}</div>
          <div className="row__control">
            <span className="row__hint">{t.read.speedUnsupported}</span>
          </div>
        </div>
      )}

      <div className="row">
        <div>
          <div className="row__label">{t.read.maxChars}</div>
          <span className="row__hint">{t.read.maxCharsHint}</span>
        </div>
        <div className="row__control">
          <input
            type="number"
            min={100}
            step={100}
            value={settings.tts.max_chars}
            onChange={(e) =>
              onSettingsChange({
                ...settings,
                tts: { ...settings.tts, max_chars: Number(e.target.value) || 0 },
              })
            }
            style={{ width: 100 }}
          />
        </div>
      </div>
    </section>
  );
}

/** Keeps a speed carried over from another provider/model inside what this one accepts. */
function clamp(speed: number, model: TtsModel): number {
  return Math.min(model.speed_max, Math.max(model.speed_min, speed));
}

/**
 * Buckets voices into `<optgroup>`s by language, preserving the provider's own order
 * within each. The unlabelled bucket (providers that don't tag voices with a language)
 * sorts last so a grouped list never opens on a heading-less run of entries.
 */
function groupByLanguage(voices: TtsVoice[]): [string, TtsVoice[]][] {
  const groups = new Map<string, TtsVoice[]>();
  for (const voice of voices) {
    const key = voice.language ?? "";
    const existing = groups.get(key);
    if (existing) existing.push(voice);
    else groups.set(key, [voice]);
  }
  return [...groups.entries()].sort(([a], [b]) => {
    if (!a) return 1;
    if (!b) return -1;
    return a.localeCompare(b);
  });
}

/**
 * Vbee's two-part credential, entered and saved as one unit.
 *
 * The app id is an ordinary settings field and the token is a vault secret, but from
 * the user's side they are one credential from one page of Vbee's console — saving
 * them separately (and on different triggers) is how "nhập key không lưu" happens.
 */
function VbeeCredentials({
  appId,
  configured,
  onSave,
}: {
  appId: string;
  configured?: boolean;
  onSave: (appId: string, token: string) => Promise<void>;
}) {
  const t = useT();
  const [draftAppId, setDraftAppId] = useState(appId);
  const [token, setToken] = useState("");
  const [saving, setSaving] = useState(false);

  // The saved app id arrives asynchronously with the rest of settings; adopt it until
  // the field has been edited, so it does not sit empty over a value that exists.
  useEffect(() => {
    setDraftAppId((current) => (current ? current : appId));
  }, [appId]);

  const save = async () => {
    setSaving(true);
    try {
      await onSave(draftAppId.trim(), token.trim());
      setToken("");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="keyline keyline--stack">
      <input
        value={draftAppId}
        onChange={(e) => setDraftAppId(e.target.value)}
        placeholder={t.read.appId}
        autoComplete="off"
      />
      <input
        type="password"
        autoComplete="off"
        placeholder={configured ? t.providers.keySaved : t.providers.keyPlaceholder}
        value={token}
        onChange={(e) => setToken(e.target.value)}
      />
      <button onClick={save} disabled={saving || !draftAppId.trim() || !token.trim()}>
        {t.common.save}
      </button>
    </div>
  );
}
