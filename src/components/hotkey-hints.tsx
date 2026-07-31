/**
 * The two ways to start a dictation, spelled out.
 *
 * A key name on its own does not say how to use it, and push-to-talk is the one binding
 * where that matters: shown as a bare `F9`, everyone presses it once and lets go, which
 * starts and immediately ends a take. The verb belongs next to the key, everywhere the
 * key appears — so the wording lives here once rather than being re-invented per screen.
 */

import { formatAccelerator, pushToTalkLabel } from "../lib/format-accelerator";
import { useT } from "../lib/i18n";
import type { HotkeySettings } from "../lib/types";

export function HotkeyHints({ hotkeys }: { hotkeys: HotkeySettings }) {
  const t = useT();
  const hold = pushToTalkLabel(hotkeys);
  const toggle = formatAccelerator(hotkeys.toggle);
  const cancel = formatAccelerator(hotkeys.cancel);

  if (!hold && !toggle && !cancel) return null;

  return (
    <div className="keyhints">
      {/* Push-to-talk first: it is the faster of the two and the one people reach for
          once they know it exists. */}
      {hold && (
        <span className="keyhints__item">
          <kbd>{hold}</kbd>
          <span>{t.dictate.holdHint}</span>
        </span>
      )}
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
