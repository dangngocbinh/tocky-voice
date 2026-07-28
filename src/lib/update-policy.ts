/** What "install the update" means on this OS — kept out of the UI so the platform
 *  branch lives in exactly one place. */

import { isMac } from "./platform";

/**
 * macOS never self-installs. This app is not code-signed (see README), so macOS
 * identifies it by cdhash, which changes on every rebuild — replacing the `.app` in
 * place would invalidate the Accessibility grant while System Settings keeps showing
 * it as "on", silently killing the app's core feature (paste) with no visible cause.
 * The same trap is already documented for manual rebuilds at README.en.md:118-121.
 * Revisit this the day the app ships with an Apple Developer ID certificate.
 */
export const canSelfInstall = () => !isMac;

/** Manual-download fallback for every platform and every failure path. A constant,
 *  never taken from the (signed but still untrusted-for-navigation) update manifest. */
export const RELEASES_URL =
  "https://github.com/dangngocbinh/tocky-voice/releases/latest";
