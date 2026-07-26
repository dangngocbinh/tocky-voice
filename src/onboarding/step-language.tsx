/** Step 1: pick the interface language, and see it change immediately. */

import { useT } from "../lib/i18n";
import type { UiLanguage } from "../lib/i18n";

const CHOICES: { value: UiLanguage; label: string; hint: string }[] = [
  { value: "system", label: "Follow system", hint: "Theo hệ thống" },
  { value: "en", label: "English", hint: "English" },
  { value: "vi", label: "Tiếng Việt", hint: "Tiếng Việt" },
];

export function StepLanguage({
  value,
  onChange,
}: {
  value: UiLanguage;
  onChange: (value: UiLanguage) => void;
}) {
  const t = useT();

  return (
    <>
      <p className="onb__tagline">{t.onboarding.tagline}</p>
      <p className="onb__lede">{t.onboarding.languageBody}</p>
      <div className="onb__choices">
        {CHOICES.map((choice) => (
          <button
            key={choice.value}
            className={`onb__choice ${value === choice.value ? "onb__choice--on" : ""}`}
            onClick={() => onChange(choice.value)}
          >
            {/* Labels stay untranslated on purpose: someone who cannot read the current
                language still has to be able to find their own. */}
            <span className="onb__choice-name">{choice.label}</span>
            {choice.value === "system" && <span className="onb__choice-note">{choice.hint}</span>}
          </button>
        ))}
      </div>
    </>
  );
}
