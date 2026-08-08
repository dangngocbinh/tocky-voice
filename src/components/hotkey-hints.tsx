/**
 * The keys a dictation is driven with, spelled out.
 *
 * A key name on its own does not say what it does — shown as a bare `⌘/`, nobody can
 * tell whether it starts a take or ends one. The verb belongs next to the key
 * everywhere the key appears, so the wording lives here once rather than being
 * re-invented per screen.
 */

import { formatAccelerator } from "../lib/format-accelerator";
import { useT } from "../lib/i18n";
import type { HotkeySettings } from "../lib/types";

export function HotkeyHints({ hotkeys }: { hotkeys: HotkeySettings }) {
  const t = useT();
  const toggle = formatAccelerator(hotkeys.toggle);
  const cancel = formatAccelerator(hotkeys.cancel);

  if (!toggle && !cancel) return null;

  return (
    <div className="keyhints">
      {toggle && (
        <span className="keyhints__item">
          <kbd>{toggle}</kbd>
          <span>{t.dictate.pressHint}</span>
        </span>
      )}
      {cancel && (
        <span className="keyhints__item">
          <kbd>{cancel}</kbd>
          <span>{t.dictate.cancelHint}</span>
        </span>
      )}
    </div>
  );
}
