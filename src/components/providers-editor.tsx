/**
 * Speech and AI provider configuration, including credentials.
 *
 * Keys are write-only from here: the UI can save one and ask whether one exists, but
 * there is no command that reads a key back across the IPC boundary.
 */

import { useCallback, useEffect, useState } from "react";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import type { AppSettings, LlmPreset, SttProviderKind } from "../lib/types";
import { STT_PROVIDERS } from "../lib/types";
import { LANGUAGES } from "../lib/languages";
import { SttBadges, useSttNote } from "./stt-provider-badges";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function ProvidersEditor({ settings, onSettingsChange }: Props) {
  const [presets, setPresets] = useState<LlmPreset[]>([]);
  const [keys, setKeys] = useState<Record<string, boolean>>({});
  const [test, setTest] = useState<{ busy: boolean; result: string | null }>({
    busy: false,
    result: null,
  });

  const t = useT();

  const refreshKeys = () => api.keyStatus().then(setKeys).catch(() => undefined);

  useEffect(() => {
    api.listLlmPresets().then(setPresets).catch(() => setPresets([]));
    refreshKeys();
  }, []);

  const active = presets.find((p) => p.id === settings.llm.preset);

  const selectPreset = (id: string) => {
    const preset = presets.find((p) => p.id === id);
    onSettingsChange({
      ...settings,
      llm: {
        ...settings.llm,
        preset: id,
        model: preset?.default_model || settings.llm.model,
        base_url: id === "custom" ? settings.llm.base_url : null,
      },
    });
  };

  const runTest = async () => {
    setTest({ busy: true, result: null });
    try {
      const reply = await api.testLlm();
      setTest({ busy: false, result: `OK — replied “${reply.slice(0, 60)}”` });
    } catch (e) {
      setTest({ busy: false, result: `Failed: ${e}` });
    }
  };

  return (
    <>
      <h1 className="view__title">{t.providers.title}</h1>
      <p className="view__lede">
{t.providers.lede}
      </p>

      <section className="section">
        <h2 className="section__title">{t.providers.speechSection}</h2>
        <div className="picker" style={{ marginTop: 14 }}>
          {STT_PROVIDERS.map((p) => (
            <label key={p.id} className={`pick ${settings.stt.provider === p.id ? "pick--on" : ""}`}>
              <input
                className="pick__radio"
                type="radio"
                name="stt"
                checked={settings.stt.provider === p.id}
                onChange={() =>
                  onSettingsChange({
                    ...settings,
                    stt: { ...settings.stt, provider: p.id as SttProviderKind },
                  })
                }
              />
              <div>
                <div className="pick__name">{p.label}</div>
                <SttBadges provider={p} />
                <SttNote provider={p} />
                <KeyField account={p.secret} configured={keys[p.secret]} onSaved={refreshKeys} />
              </div>
              <div style={{ display: "grid", gap: 6, justifyItems: "end" }}>
                <span className={`chip ${keys[p.secret] ? "chip--ok" : "chip--todo"}`}>
                  {keys[p.secret] ? t.providers.keySet : t.providers.noKey}
                </span>
                {/* Without a key the app does nothing, so the way to get one belongs
                    next to the empty field, not buried in a readme. */}
                {!keys[p.secret] && (
                  <button
                    className="btn-quiet"
                    onClick={(e) => {
                      e.preventDefault();
                      api.openUrl(p.signupUrl).catch(() => undefined);
                    }}
                  >
                    {t.providers.getKey}
                  </button>
                )}
              </div>
            </label>
          ))}
        </div>

        <div className="row" style={{ marginTop: 14 }}>
          <div>
            <div className="row__label">{t.providers.language}</div>
            <span className="row__hint">{t.providers.languageHint}</span>
          </div>
          <div className="row__control">
            <select
              value={settings.stt.language}
              onChange={(e) =>
                onSettingsChange({
                  ...settings,
                  stt: { ...settings.stt, language: e.target.value },
                })
              }
            >
              {LANGUAGES.map((l) => (
                <option key={l.code} value={l.code}>
                  {l.label} ({l.code})
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="row row--stack">
          <div className="row__label">
            {t.providers.hints}
            <span className="row__hint">{t.providers.hintsHint}</span>
          </div>
          <div className="tags">
            {LANGUAGES.map((l) => {
              const index = settings.stt.language_hints.indexOf(l.code);
              const on = index >= 0;
              return (
                <button
                  key={l.code}
                  className={`tag-toggle ${on ? "tag-toggle--on" : ""}`}
                  onClick={() =>
                    onSettingsChange({
                      ...settings,
                      stt: {
                        ...settings.stt,
                        language_hints: on
                          ? settings.stt.language_hints.filter((c) => c !== l.code)
                          : [...settings.stt.language_hints, l.code],
                      },
                    })
                  }
                >
                  {on && <span className="tag-order">{index + 1}</span>}
                  {l.label}
                </button>
              );
            })}
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.providers.aiSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.providers.provider}</div>
<span className="row__hint">{t.providers.providerHint}</span>
          </div>
          <div className="row__control">
            <select value={settings.llm.preset} onChange={(e) => selectPreset(e.target.value)}>
              {presets.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.label}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.providers.model}</div>
<span className="row__hint">{t.providers.modelHint}</span>
          </div>
          <div className="row__control">
            <ModelPicker
              fallback={active?.models ?? []}
              preset={settings.llm.preset}
              value={settings.llm.model}
              onChange={(model) =>
                onSettingsChange({ ...settings, llm: { ...settings.llm, model } })
              }
            />
          </div>
        </div>

        {settings.llm.preset === "custom" && (
          <div className="row row--stack">
            <div className="row__label">
              {t.providers.baseUrl}
              <span className="row__hint">{t.providers.baseUrlHint}</span>
            </div>
            <input
              placeholder="https://your-endpoint/v1"
              value={settings.llm.base_url ?? ""}
              onChange={(e) =>
                onSettingsChange({
                  ...settings,
                  llm: { ...settings.llm, base_url: e.target.value || null },
                })
              }
            />
          </div>
        )}

        {active?.needs_key && (
          <div className="row row--stack">
            <div className="row__label">
              {t.providers.apiKey}
              <span className="row__hint">
                {t.providers.apiKeyHint}{" "}
                {active.signup_url && (
                  <a
                    href="#"
                    onClick={(e) => {
                      e.preventDefault();
                      api.openUrl(active.signup_url).catch(() => undefined);
                    }}
                  >
                    {t.providers.getKeyFrom} {active.label} ↗
                  </a>
                )}
              </span>
            </div>
            <KeyField
              account={active.secret_key}
              configured={keys[active.secret_key]}
              onSaved={refreshKeys}
            />
          </div>
        )}

        <div className="row row--tight">
          <div className="row__label">{t.providers.testSection}</div>
          <div className="row__control">
            {test.result && <span className="muted" style={{ fontSize: 11.5 }}>{test.result}</span>}
            <button onClick={runTest} disabled={test.busy}>
              {test.busy ? t.providers.testing : t.providers.test}
            </button>
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.providers.storageSection}</h2>
        <div className="row">
          <div>
            <div className="row__label">{t.providers.useKeychain}</div>
<span className="row__hint">{t.providers.useKeychainHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.use_os_keychain}
              onChange={(v) => onSettingsChange({ ...settings, use_os_keychain: v })}
            />
          </div>
        </div>
      </section>
    </>
  );
}

/**
 * Model list fetched live from the provider, so a model released today is selectable
 * today. Falls back to the preset's built-in list when the fetch fails — no key saved
 * yet, offline, or a provider without a `/models` endpoint.
 */
function ModelPicker({
  fallback,
  preset,
  value,
  onChange,
}: {
  fallback: string[];
  preset: string;
  value: string;
  onChange: (model: string) => void;
}) {
  const t = useT();
  const [models, setModels] = useState<string[]>(fallback);
  const [live, setLive] = useState(false);
  const [loading, setLoading] = useState(false);
  const [custom, setCustom] = useState(false);

  const load = useCallback(() => {
    setLoading(true);
    api
      .listModels()
      .then((list) => {
        setModels(list);
        setLive(true);
      })
      .catch(() => {
        setModels(fallback);
        setLive(false);
      })
      .finally(() => setLoading(false));
  }, [fallback]);

  // Re-fetch when the provider changes; its model list is completely different.
  useEffect(() => {
    setModels(fallback);
    setLive(false);
    load();
  }, [preset]); // eslint-disable-line react-hooks/exhaustive-deps

  if (custom) {
    return (
      <>
        <input value={value} onChange={(e) => onChange(e.target.value)} placeholder="model-id" />
        <button className="btn-quiet" onClick={() => setCustom(false)}>
          {t.common.list}
        </button>
      </>
    );
  }

  // The saved model may not be in the list (renamed, or set before a provider switch).
  const options = models.includes(value) ? models : [value, ...models].filter(Boolean);

  return (
    <>
      <select
        value={value}
        onChange={(e) =>
          e.target.value === "__custom__" ? setCustom(true) : onChange(e.target.value)
        }
      >
        {options.map((m) => (
          <option key={m} value={m}>
            {m}
          </option>
        ))}
        <option value="__custom__">{t.common.other}</option>
      </select>
      <button
        className="btn-quiet"
        onClick={load}
        disabled={loading}
        title={live ? "Fetched from the provider" : "Built-in list — click to fetch live"}
      >
        {loading ? "…" : live ? "↻" : t.common.fetch}
      </button>
    </>
  );
}

export function Switch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className="switch">
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
      <span className="switch__track" />
    </label>
  );
}

function KeyField({
  account,
  configured,
  onSaved,
}: {
  account: string;
  configured?: boolean;
  onSaved: () => void;
}) {
  const t = useT();
  const [value, setValue] = useState("");
  const [saving, setSaving] = useState(false);

  const save = async () => {
    setSaving(true);
    try {
      await api.setApiKey(account, value);
      setValue("");
      onSaved();
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="keyline" onClick={(e) => e.preventDefault()}>
      <input
        type="password"
        autoComplete="off"
        placeholder={configured ? t.providers.keySaved : t.providers.keyPlaceholder}
        value={value}
        onChange={(e) => setValue(e.target.value)}
      />
      <button onClick={save} disabled={saving || !value}>
        {t.common.save}
      </button>
      {configured && (
        <button className="btn-quiet btn-danger" onClick={() => api.deleteApiKey(account).then(onSaved)}>
          {t.common.remove}
        </button>
      )}
    </div>
  );
}

/** Wrapper so the note can call the translation hook from inside the provider list. */
function SttNote({ provider }: { provider: (typeof STT_PROVIDERS)[number] }) {
  return <p className="pick__note">{useSttNote(provider)}</p>;
}
