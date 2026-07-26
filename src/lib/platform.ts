/**
 * Which OS the webview is running on.
 *
 * Read from the user agent rather than a plugin: the only thing the UI needs this for
 * is which key names to print and which setup steps apply, and that does not justify a
 * dependency. `navigator.platform` is deprecated, so the user agent string it is.
 */

const ua = typeof navigator === "undefined" ? "" : navigator.userAgent;

export const isMac = /Mac(intosh| OS X)/i.test(ua);
export const isWindows = /Windows/i.test(ua);
