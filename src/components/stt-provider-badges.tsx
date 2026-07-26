/**
 * The badge that tells someone why they would pick a given speech provider.
 *
 * Exactly one badge each, because each provider has exactly one reason to be chosen:
 *
 *   Soniox      → Best for Vietnamese   (most accurate on mixed VI/EN; no free credit)
 *   Deepgram    → Free $200             (largest free allowance; ~690 hours)
 *   AssemblyAI  → Free $50              (English only on streaming)
 *
 * A row of four chips per provider says nothing — the point of a badge is that it can
 * be read at a glance while scanning a list.
 *
 * Shared by the onboarding step, the Providers tab and the About tab so the three can
 * never describe the same provider differently.
 */

import { useT } from "../lib/i18n";
import type { STT_PROVIDERS } from "../lib/types";

type Provider = (typeof STT_PROVIDERS)[number];

export function SttBadge({ provider }: { provider: Provider }) {
  const t = useT();

  const label =
    provider.badge === "free_credit"
      ? t.stt.free_credit.replace("{amount}", provider.freeCredit ?? "")
      : t.stt.best_vietnamese;

  // Green reads as "costs you nothing to try", amber as "this is the good one" — the
  // two reasons are different, so they must not share a colour.
  const tone = provider.badge === "free_credit" ? "chip--ok" : "chip--star";

  return <span className={`chip ${tone}`}>{label}</span>;
}

/** The one-line description, in the user's language. */
export function useSttNote(provider: Provider): string {
  const t = useT();
  return t.stt[provider.id as "soniox" | "deepgram" | "assembly_ai"];
}
