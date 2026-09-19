/** Step 1: pick the interface language and the theme, and see both change immediately. */

import type { ReactNode } from "react";
import { useT } from "../lib/i18n";
import type { UiLanguage } from "../lib/i18n";
import type { UiTheme } from "../lib/theme";
import {
  MonitorIcon,
  MoonIcon,
  SunIcon,
  UnionFlag,
  VietnamFlag,
} from "../components/icons";

const LANGUAGES: { value: UiLanguage; label: string; hint: string; icon: ReactNode }[] = [
  {
    value: "system",
    label: "Follow system",
    hint: "Theo hệ thống",
    icon: <MonitorIcon className="onb__choice-icon" />,
  },
  { value: "en", label: "English", hint: "English", icon: <UnionFlag className="onb__flag" /> },
  {
    value: "vi",
    label: "Tiếng Việt",
    hint: "Tiếng Việt",
    icon: <VietnamFlag className="onb__flag" />,
  },
];

export function StepLanguage({
  value,
  onChange,
  theme,
  onThemeChange,
}: {
  value: UiLanguage;
  onChange: (value: UiLanguage) => void;
  theme: UiTheme;
  onThemeChange: (theme: UiTheme) => void;
}) {
  const t = useT();

  // Translated, unlike the language labels: by the time anyone reads this they have
  // already chosen a language they can read.
  const themes: { value: UiTheme; label: string; icon: ReactNode }[] = [
    {
      value: "system",
      label: t.behaviour.followSystem,
      icon: <MonitorIcon className="onb__choice-icon" />,
    },
    { value: "light", label: t.behaviour.themeLight, icon: <SunIcon className="onb__choice-icon" /> },
    { value: "dark", label: t.behaviour.themeDark, icon: <MoonIcon className="onb__choice-icon" /> },
  ];

  return (
    <>
      <p className="onb__tagline">{t.onboarding.tagline}</p>
      <p className="onb__lede">{t.onboarding.languageBody}</p>
      <div className="onb__choices">
        {LANGUAGES.map((choice) => (
          <button
            key={choice.value}
            className={`onb__choice ${value === choice.value ? "onb__choice--on" : ""}`}
            onClick={() => onChange(choice.value)}
          >
            {/* Labels stay untranslated on purpose: someone who cannot read the current
                language still has to be able to find their own — and the flag beside
                them is findable without reading anything at all. */}
            <span className="onb__choice-name">
              {choice.icon}
              {choice.label}
            </span>
            {choice.value === "system" && <span className="onb__choice-note">{choice.hint}</span>}
          </button>
        ))}
      </div>

      {/* Here rather than buried in Settings, because this is the screen the app is
          judged on — and the whole choice is visible the instant it is made. */}
      <p className="onb__prompt">{t.onboarding.themePrompt}</p>
      <div className="onb__choices onb__choices--row">
        {themes.map((choice) => (
          <button
            key={choice.value}
            className={`onb__choice ${theme === choice.value ? "onb__choice--on" : ""}`}
            onClick={() => onThemeChange(choice.value)}
          >
            <span className="onb__choice-name">
              {choice.icon}
              {choice.label}
            </span>
          </button>
        ))}
      </div>
    </>
  );
}
