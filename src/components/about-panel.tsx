/**
 * About: who made this, and where to get the API keys it needs.
 *
 * The signup list matters more than the credits: someone installing this has an app
 * that does nothing until they hold accounts at two separate vendors. Putting the
 * links here — with what each one actually costs — is the difference between the app
 * working on day one and being abandoned at the first empty key field.
 */

import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import { STT_PROVIDERS } from "../lib/types";
import { SttBadges, useSttNote } from "./stt-provider-badges";
import type { LlmPreset } from "../lib/types";
import mecodeLogo from "../assets/brand/mecode-logo.png";

interface Props {
  presets: LlmPreset[];
}

/** Free allowances stated by the vendor. Anything unverified is left out on purpose. */
const LLM_NOTES: Record<string, string> = {
  gemini: "Free tier in AI Studio — generous daily quota, no card needed",
  groq: "Free tier, and the fastest of the hosted options",
  ollama: "Runs on your own machine — no account, no cost, no data leaves the laptop",
  deepseek: "Pay as you go, very cheap — but it reasons, so expect a few seconds",
  openrouter: "One key, most models — handy if you want to compare",
};

export function AboutPanel({ presets }: Props) {
  const t = useT();
  const open = (url: string) => api.openUrl(url).catch(() => undefined);

  return (
    <>
      <h1 className="view__title">{t.about.title}</h1>
      <p className="view__tagline">{t.onboarding.tagline}</p>
      <p className="view__lede">
{t.about.lede}
      </p>

      <section className="section">
        <h2 className="section__title">{t.about.madeBy}</h2>
        <div className="author">
          <img className="author__logo" src={mecodeLogo} alt="ME Code" width={64} height={64} />
          <div>
            <div className="author__name">ME Code</div>
            <p className="author__tag">
              AI Automation Academy &amp; Product Lab — dạy và xây sản phẩm ứng dụng AI,
              VibeCode, AI Agent và tự động hoá.
            </p>
            <div className="author__links">
              <button className="btn-quiet" onClick={() => open("https://mecode.pro")}>
                mecode.pro ↗
              </button>
              <span className="muted">hello@mecode.pro</span>
              <span className="muted">0965 095 174</span>
            </div>
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.about.sttSection}</h2>
        <p className="row__hint" style={{ marginTop: 10 }}>
{t.about.sttHint}
        </p>
        {STT_PROVIDERS.map((p) => (
          <div className="row" key={p.id}>
            <div>
              <div className="row__label">{p.label}</div>
              <SttBadges provider={p} />
              <SttNote provider={p} />
            </div>
            <div className="row__control">
              <button onClick={() => open(p.signupUrl)}>{t.about.signUp}</button>
            </div>
          </div>
        ))}
      </section>

      <section className="section">
        <h2 className="section__title">{t.about.llmSection}</h2>
        <p className="row__hint" style={{ marginTop: 10 }}>
{t.about.llmHint}
        </p>
        {presets
          .filter((p) => p.id !== "custom")
          .map((p) => (
            <div className="row row--tight" key={p.id}>
              <div>
                <div className="row__label">{p.label}</div>
                {LLM_NOTES[p.id] && <span className="row__hint">{LLM_NOTES[p.id]}</span>}
              </div>
              <div className="row__control">
                <button className="btn-quiet" onClick={() => open(p.signup_url)}>
                  {t.about.getKey}
                </button>
              </div>
            </div>
          ))}
      </section>

      <section className="section">
        <h2 className="section__title">{t.about.beforeSection}</h2>
        <div className="row row--stack">
          <span className="row__hint">
{t.about.privacy}
          </span>
        </div>
      </section>
    </>
  );
}

/** Wrapper so the note can call the translation hook from inside the provider list. */
function SttNote({ provider }: { provider: (typeof STT_PROVIDERS)[number] }) {
  return <span className="row__hint">{useSttNote(provider)}</span>;
}
