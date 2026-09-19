/**
 * Interface theme.
 *
 * The preference is stored as `system | light | dark` and resolved here, in the one
 * place that can see `prefers-color-scheme`. Applying it means a single attribute on
 * `<html>`; every colour in `styles.css` is a token, and `[data-theme="light"]`
 * restates the tokens — so nothing else in the app has to know which theme is on.
 */

import { useEffect } from "react";

export type UiTheme = "system" | "light" | "dark";

const DARK_QUERY = "(prefers-color-scheme: dark)";

/** Resolve `system` against whatever the OS is currently set to. */
export function resolveTheme(preference: UiTheme): "light" | "dark" {
  if (preference !== "system") return preference;
  return window.matchMedia?.(DARK_QUERY).matches ? "dark" : "light";
}

/**
 * Keeps `<html data-theme>` in step with the preference, following the OS live while
 * the preference is `system` — someone whose Mac switches to dark at sunset should not
 * have to restart the app to see it.
 */
export function useTheme(preference: UiTheme): void {
  useEffect(() => {
    const apply = () => {
      document.documentElement.dataset.theme = resolveTheme(preference);
    };
    apply();

    if (preference !== "system") return;
    const media = window.matchMedia?.(DARK_QUERY);
    if (!media) return;
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [preference]);
}
