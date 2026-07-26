/**
 * Step 3: Accessibility permission.
 *
 * Granting happens in System Settings, in another app — so the only way this step can
 * feel finished is if it notices on its own and turns green. Polling is what makes the
 * user's action visibly land instead of leaving them wondering whether it worked.
 */

import { useEffect, useState } from "react";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import { usePermissionStatus } from "../lib/use-permission-status";

/** Long enough for the label change to register as a response to the click. */
const FEEDBACK_MS = 700;

export function StepPermission({ onReadyChange }: { onReadyChange: (ready: boolean) => void }) {
  const { accessibility, refresh } = usePermissionStatus();
  const [checking, setChecking] = useState(false);
  const t = useT();

  useEffect(() => onReadyChange(accessibility), [accessibility, onReadyChange]);

  // A check that comes back in 5ms looks like a button that does nothing. Holding the
  // "checking" state briefly is the difference between "it ignored me" and "it looked,
  // and the answer is still no".
  const recheck = () => {
    setChecking(true);
    const done = new Promise((r) => window.setTimeout(r, FEEDBACK_MS));
    Promise.all([refresh(), done]).finally(() => setChecking(false));
  };

  return (
    <>
      <p className="onb__lede">{t.onboarding.permissionBody}</p>

      <div className={`onb__status ${accessibility ? "onb__status--ok" : ""}`}>
        <span className="onb__status-dot" />
        <span>{accessibility ? t.onboarding.permissionGranted : t.onboarding.permissionMissing}</span>
      </div>

      {!accessibility && (
        <>
          <ol className="onb__list">
            <li>{t.onboarding.permissionStep1}</li>
            <li>{t.onboarding.permissionStep2}</li>
            <li>{t.onboarding.permissionStep3}</li>
            <li>{t.onboarding.permissionStep4}</li>
          </ol>
          <div className="onb__actions">
            <button
              className="btn-primary"
              onClick={() => api.openAccessibilitySettings().catch(() => undefined)}
            >
              {t.common.openSettings}
            </button>
            <button className="btn-quiet" onClick={recheck} disabled={checking}>
              {checking ? t.onboarding.checking : t.onboarding.recheck}
            </button>
          </div>
        </>
      )}
    </>
  );
}
