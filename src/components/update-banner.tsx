/** Dismissible "a new version is out" strip, shown above the active section. */

import { useT } from "../lib/i18n";

interface Props {
  version: string;
  onSeeWhatsNew: () => void;
  onDismiss: () => void;
}

export function UpdateBanner({ version, onSeeWhatsNew, onDismiss }: Props) {
  const t = useT();

  return (
    <div className="notice notice--warn">
      <span>
        {t.update.newVersionAvailable} v{version}
      </span>
      <div className="row__control">
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
