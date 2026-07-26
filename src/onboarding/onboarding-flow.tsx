/**
 * First-run walkthrough.
 *
 * The app is useless until three things are true: a speech provider has a key, macOS
 * has granted Accessibility, and the user knows the hotkey. Each of those fails
 * silently in its own way, so rather than leaving people to discover them one broken
 * dictation at a time, this walks through them in order and refuses to advance until
 * each one actually works.
 *
 * Re-runnable on purpose — it doubles as the "why has it stopped working" checklist.
 */

import { useState } from "react";
import { useT } from "../lib/i18n";
import type { UiLanguage } from "../lib/i18n";
import { isMac } from "../lib/platform";
import type { AppSettings } from "../lib/types";
import { StepLanguage } from "./step-language";
import { StepSpeech } from "./step-speech";
import { StepPermission } from "./step-permission";
import { StepTryIt } from "./step-try-it";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
  onDone: () => void;
}

// Accessibility is a macOS concept. Showing a step that is already green and talks
// about System Settings would be four steps of setup where three are real.
const STEPS = (
  isMac
    ? (["language", "speech", "permission", "try"] as const)
    : (["language", "speech", "try"] as const)
) as readonly StepId[];

export type StepId = "language" | "speech" | "permission" | "try";

export function OnboardingFlow({ settings, onSettingsChange, onDone }: Props) {
  const [index, setIndex] = useState(0);
  const [canAdvance, setCanAdvance] = useState(true);
  const t = useT();

  const step = STEPS[index];
  const last = index === STEPS.length - 1;

  const titles: Record<StepId, string> = {
    language: t.onboarding.languageTitle,
    speech: t.onboarding.speechTitle,
    permission: t.onboarding.permissionTitle,
    try: t.onboarding.tryTitle,
  };

  const finish = () => {
    onSettingsChange({ ...settings, onboarding_completed: true });
    onDone();
  };

  const next = () => {
    if (last) finish();
    else setIndex((i) => i + 1);
  };

  return (
    <div className="onb">
      <div className="onb__panel">
        <header className="onb__head">
          <div className="onb__dots">
            {STEPS.map((id, i) => (
              <span
                key={id}
                className={`onb__dot ${i === index ? "onb__dot--on" : ""} ${
                  i < index ? "onb__dot--done" : ""
                }`}
              />
            ))}
          </div>
          <h1 className="onb__title">{titles[step]}</h1>
          <span className="onb__count">
            {index + 1} / {STEPS.length}
          </span>
        </header>

        <div className="onb__body">
          {step === "language" && (
            <StepLanguage
              value={settings.ui_language}
              onChange={(ui_language: UiLanguage) =>
                onSettingsChange({ ...settings, ui_language })
              }
            />
          )}
          {step === "speech" && (
            <StepSpeech
              settings={settings}
              onSettingsChange={onSettingsChange}
              onReadyChange={setCanAdvance}
            />
          )}
          {step === "permission" && <StepPermission onReadyChange={setCanAdvance} />}
          {step === "try" && <StepTryIt settings={settings} onSettingsChange={onSettingsChange} />}
        </div>

        <footer className="onb__foot">
          {/* Escape hatch. Someone who already knows the app should never be trapped
              behind four screens, and the flow can be re-run from Settings. */}
          <button className="btn-quiet" onClick={finish}>
            {t.onboarding.skip}
          </button>
          <div className="onb__nav">
            {index > 0 && (
              <button
                className="btn-quiet"
                onClick={() => {
                  setCanAdvance(true);
                  setIndex((i) => i - 1);
                }}
              >
                {t.onboarding.back}
              </button>
            )}
            <button className="btn-primary" onClick={next} disabled={!canAdvance}>
              {last ? t.onboarding.finish : t.onboarding.next}
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
}
