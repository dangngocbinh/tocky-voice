/** The About tab's "Updates" section: version, check button, notes, install action. */

import { useT } from "../lib/i18n";
import type { UseUpdateCheck } from "../lib/use-update-check";
import { canSelfInstall } from "../lib/update-policy";

interface Props {
  version: string | null;
  update: UseUpdateCheck;
}

export function UpdateStatusSection({ version, update }: Props) {
  const t = useT();

  return (
    <section className="section">
      <h2 className="section__title">{t.update.sectionTitle}</h2>

      <div className="row">
        <div className="row__label">{t.update.currentVersion}</div>
        <div className="row__control mono">{version ? `v${version}` : "…"}</div>
      </div>

      <div className="row">
        <div>
          {update.state === "checking" && <span className="row__hint">{t.update.checking}</span>}
          {update.state === "none" && <span className="row__hint">{t.update.upToDate}</span>}
          {update.state === "error" && (
            <span className="row__hint">
              {t.update.errorPrefix} {update.errorMessage}
            </span>
          )}
          {(update.state === "available" ||
            update.state === "downloading" ||
            update.state === "installed") &&
            update.update && (
              <span className="row__hint">
                {t.update.newVersionAvailable} v{update.update.version}
              </span>
            )}
        </div>
        <div className="row__control">
          <button
            className="btn-quiet"
            disabled={update.state === "checking" || update.state === "downloading"}
            onClick={update.check}
          >
            {t.update.checkButton}
          </button>
        </div>
      </div>

      {update.update?.body && (
        <div className="row row--stack">
          <span className="row__label">{t.update.notesTitle}</span>
          <p className="notes">{update.update.body}</p>
        </div>
      )}

      {update.state === "available" && (
        <div className="row">
          {!canSelfInstall() && <span className="row__hint">{t.update.macOnlyReason}</span>}
          <div className="row__control">
            <button onClick={update.install}>
              {canSelfInstall() ? t.update.installButton : t.update.downloadButton}
            </button>
          </div>
        </div>
      )}

      {update.state === "downloading" && (
        <div className="row">
          <span className="row__hint mono">
            {t.update.downloading} {update.progress}%
          </span>
        </div>
      )}

      {update.state === "installed" && (
        <div className="row">
          <span className="row__hint">{t.update.installedRestarting}</span>
        </div>
      )}
    </section>
  );
}
