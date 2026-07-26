/**
 * The badges that tell someone which speech provider to pick.
 *
 * Shared by the onboarding step, the Providers tab and the About tab so the three never
 * drift apart — a provider described as "recommended" in one place and "paid only" in
 * another is worse than no label at all.
 *
 * Two badges, answering two different questions:
 *   - "Which one understands me best?"  → Best for Vietnamese
 *   - "Which one can I start on today?" → Start here (largest free credit)
 *
 * They deliberately land on different providers. Soniox is the most accurate on mixed
 * Vietnamese and English but bills from the first minute; Deepgram gives $200 free.
 * Hiding that trade-off would push people onto a provider they cannot afford to try.
 */

import { useT } from "../lib/i18n";
import type { STT_PROVIDERS } from "../lib/types";

type Provider = (typeof STT_PROVIDERS)[number];

export function SttBadges({ provider }: { provider: Provider }) {
  const t = useT();

  return (
    <span className="badges">
      {provider.bestForVietnamese && (
        <span className="chip chip--star">{t.stt.bestForVietnamese}</span>
      )}
      {provider.startHere && <span className="chip chip--ok">{t.stt.startHere}</span>}
      <span className={`chip ${provider.freeCredit ? "chip--ok" : "chip--muted"}`}>
        {provider.freeCredit
          ? t.stt.freeCredit.replace("{amount}", provider.freeCredit)
          : t.stt.noFreeCredit}
      </span>
      <span className="chip chip--muted">
        {t.stt.perHour.replace("{price}", provider.hourlyUsd.toFixed(2))}
      </span>
    </span>
  );
}

/** The one-line description, in the user's language. */
export function useSttNote(provider: Provider): string {
  const t = useT();
  return t.stt[provider.id as "soniox" | "deepgram" | "assembly_ai"];
}
