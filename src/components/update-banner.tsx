/**
 * "A new version is out" strip, shown above the active section.
 *
 * It carries the update button itself rather than only pointing at the About tab: the
 * shortest honest path from "there is a new version" to "you are running it" is one
 * click, and the progress reports back in the same strip so nobody has to go looking
 * for what happened after pressing it.
 */

import { useT } from "../lib/i18n";
import type { UseUpdateCheck } from "../lib/use-update-check";

interface Props {
  update: UseUpdateCheck;
  version: string;
  onSeeWhatsNew: () => void;
  onDismiss: () => void;
}

export function UpdateBanner({ update, version, onSeeWhatsNew, onDismiss }: Props) {
  const t = useT();

  if (update.state === "downloading") {
    return (
      <div className="notice notice--warn">
        <span className="mono">
          {t.update.downloading} {update.progress}%
        </span>
      </div>
    );
  }

  if (update.state === "installed") {
    return (
      <div className="notice notice--warn">
        <span>{t.update.installedRestarting}</span>
      </div>
    );
  }

  return (
    <div className="notice notice--warn">
      <span>
        {t.update.newVersionAvailable} v{version}
      </span>
      <div className="row__control">
        <button className="btn-primary" onClick={update.install}>
          {update.canInstallInPlace ? t.update.updateNow : t.update.downloadButton}
        </button>
        <button className="btn-quiet" onClick={onSeeWhatsNew}>
          {t.update.bannerSeeWhatsNew}
        </button>
        <button className="btn-quiet" onClick={onDismiss}>
          {t.update.bannerDismiss}
        </button>
      </div>
    </div>
  );
}
