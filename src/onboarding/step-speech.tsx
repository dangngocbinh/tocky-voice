/**
 * The speech provider and its key.
 *
 * This is the one step that cannot be skipped — without a key the app transcribes
 * nothing, and the failure at dictation time is a toast that vanishes. So the key is
 * saved and confirmed here, before anyone tries to use the app for real.
 *
 * "Confirmed" means the key was used, not merely stored. A saved key proves only that
 * a string reached disk: a typo, a revoked key or an account out of credit all looked
 * exactly like success and only surfaced later as dictation that produced nothing. So
 * saving opens a real stream against the provider, and the step stays closed until the
 * provider itself accepts the credential.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import type { AppSettings, SttProviderKind } from "../lib/types";
import { STT_PROVIDERS } from "../lib/types";
import { SttBadge, useSttNote } from "../components/stt-provider-badges";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
  onReadyChange: (ready: boolean) => void;
}

/** Where the key stands with the provider, as opposed to with the disk. */
type Check =
  | { state: "unknown" }
  | { state: "checking" }
  | { state: "ok" }
  | { state: "failed"; detail: string };

export function StepSpeech({ settings, onSettingsChange, onReadyChange }: Props) {
  const [keys, setKeys] = useState<Record<string, boolean>>({});
  const [value, setValue] = useState("");
  const [saving, setSaving] = useState(false);
  const [check, setCheck] = useState<Check>({ state: "unknown" });
  const t = useT();

  const active = STT_PROVIDERS.find((p) => p.id === settings.stt.provider) ?? STT_PROVIDERS[0];
  const configured = !!keys[active.secret];

  const refresh = () => api.keyStatus().then(setKeys).catch(() => undefined);
  useEffect(() => {
    refresh();
  }, []);

  // Read through a ref so `verify` keeps a stable identity: the parent hands down a
  // fresh settings object on every save and on every push from the backend, and a
  // `verify` that changed with it would re-open a stream against the provider each
  // time one of those landed.
  const sttRef = useRef(settings.stt);
  sttRef.current = settings.stt;

  const verify = useCallback(async () => {
    setCheck({ state: "checking" });
    try {
      await api.testSttKey(sttRef.current);
      setCheck({ state: "ok" });
    } catch (e) {
      setCheck({ state: "failed", detail: String(e) });
    }
  }, []);

  // Re-check whenever the provider changes, and on arrival for a key saved earlier —
  // this screen doubles as the "why has it stopped working" checklist, and a key that
  // has since been revoked should read as broken here rather than as a green tick.
  useEffect(() => {
    if (configured) verify();
    else setCheck({ state: "unknown" });
  }, [configured, active.secret, verify]);

  // Stored is not the same as working, so the gate is the provider's own verdict.
  // Anyone who disagrees — offline, or checking a key they know is fine — has Skip.
  useEffect(() => onReadyChange(check.state === "ok"), [check.state, onReadyChange]);

  const save = async () => {
    setSaving(true);
    try {
      await api.setApiKey(active.secret, value.trim());
      setValue("");
      await refresh();
      await verify();
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
              <SttBadge provider={p} />
              {keys[p.secret] && <span className="onb__tick">✓</span>}
            </span>
            <SttNote provider={p} />
          </button>
        ))}
      </div>

      {configured && (
        <div className={`onb__status ${check.state === "ok" ? "onb__status--ok" : ""}`}>
          <span className="onb__status-dot" />
          <span>
            {check.state === "checking"
              ? t.onboarding.keyChecking.replace("{provider}", active.label)
              : check.state === "ok"
                ? t.onboarding.keyWorks.replace("{provider}", active.label)
                : check.state === "failed"
                  ? `${t.onboarding.keyRejected.replace("{provider}", active.label)} ${check.detail}`
                  : t.onboarding.keySaved.replace("{provider}", active.label)}
          </span>
        </div>
      )}

      {/* A key the provider will not accept is no better than no key, so the way back
          to the entry field stays open until one is. */}
      {(!configured || check.state === "failed") && (
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
            <SttBadge provider={active} />
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
              {saving ? t.onboarding.checking : t.common.save}
            </button>
          </div>
        </>
      )}

      {configured && check.state !== "checking" && (
        <div className="onb__actions" style={{ marginTop: 12 }}>
          <button className="btn-quiet" onClick={verify}>
            {t.onboarding.recheck}
          </button>
        </div>
      )}
    </>
  );
}

/** Small wrapper so the note can use the translation hook inside the list. */
function SttNote({ provider }: { provider: (typeof STT_PROVIDERS)[number] }) {
  return <span className="onb__choice-note">{useSttNote(provider)}</span>;
}
