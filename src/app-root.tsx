/**
 * Both windows load the same bundle; the window label decides what to render.
 * Keeping one entry point avoids a second Vite build target for a 200px panel.
 *
 * The language provider wraps both, so the overlay speaks the same language as the
 * settings window without each surface fetching and resolving it separately — and the
 * theme is applied the same way, so switching it repaints the floating panels too.
 */

import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import * as api from "./lib/api";
import { dictionaryFor, resolveLanguage, TranslationContext } from "./lib/i18n";
import type { UiLanguage } from "./lib/i18n";
import { useTheme } from "./lib/theme";
import type { UiTheme } from "./lib/theme";
import { OverlayPanel } from "./overlay-panel";
import { PlayerPanel } from "./player-panel";
import { SettingsApp } from "./settings-app";

export function AppRoot() {
  const label = getCurrentWebviewWindow().label;
  const [preference, setPreference] = useState<UiLanguage>("system");
  const [theme, setTheme] = useState<UiTheme>("system");

  useEffect(() => {
    const load = () =>
      api
        .getSettings()
        .then((s) => {
          setPreference(s.ui_language ?? "system");
          setTheme(s.theme ?? "system");
        })
        .catch(() => undefined);
    load();
    // Picking a language in Settings must retitle the overlay too, not just this
    // window — and the same goes for the theme.
    const off = listen(api.EVENTS.settingsChanged, load);
    return () => {
      off.then((fn) => fn()).catch(() => undefined);
    };
  }, []);

  useTheme(theme);

  return (
    <TranslationContext.Provider value={dictionaryFor(resolveLanguage(preference))}>
      {label === "overlay" ? (
        <OverlayPanel />
      ) : label === "player" ? (
        <PlayerPanel />
      ) : (
        <SettingsApp />
      )}
    </TranslationContext.Provider>
  );
}
