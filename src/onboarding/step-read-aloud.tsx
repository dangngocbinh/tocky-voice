/**
 * Last step of onboarding: mention read-aloud exists, without pushing anyone into it.
 * Whoever just finished setting up dictation should not be dragged into a second
 * feature — the shared footer's "Start using it" button below doubles as "Maybe
 * later" for this step and leaves `tts.enabled` untouched; the button in this step's
 * own body is the only thing that turns it on.
 */

import { formatAccelerator } from "../lib/format-accelerator";
import { useT } from "../lib/i18n";
import type { AppSettings } from "../lib/types";
import { SpeakerIcon } from "../components/icons";

interface Props {
  settings: AppSettings;
  onEnableAndFinish: () => void;
}

export function StepReadAloud({ settings, onEnableAndFinish }: Props) {
  const t = useT();
  const hotkey = formatAccelerator(settings.hotkeys.read);

  return (
    <div className="onb__read">
      <SpeakerIcon className="onb__read-icon" />
      <p className="onb__lede">{t.onboarding.readAloudBody.replace("{hotkey}", hotkey ?? "")}</p>
      <p className="onb__note">{t.onboarding.readAloudHint}</p>
      <button className="btn-primary" onClick={onEnableAndFinish}>
        {hotkey ? t.onboarding.readAloudEnable.replace("{hotkey}", hotkey) : t.read.enable}
      </button>
    </div>
  );
}
