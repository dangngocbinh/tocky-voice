/**
 * Main window shell.
 *
 * A left rail rather than top tabs: this is a settings-heavy desktop app, and the rail
 * keeps the active section visible while leaving the full width for content. It also
 * gives the window somewhere honest to show live state, so you can tell at a glance
 * whether the app is listening.
 *
 * Edits save automatically on a short debounce. Everything here is small and
 * reversible, and a hotkey change should take effect the moment you finish setting it.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import * as api from "./lib/api";
import { useDictationEvents } from "./lib/use-dictation-events";
import { useUpdateCheck } from "./lib/use-update-check";
import { useT } from "./lib/i18n";
import { OnboardingFlow } from "./onboarding/onboarding-flow";
import type { AppSettings, LlmPreset } from "./lib/types";
import { BehaviourEditor } from "./components/behaviour-editor";
import { DictationPanel } from "./components/dictation-panel";
import { HistoryList } from "./components/history-list";
import { ModesEditor } from "./components/modes-editor";
import { ProvidersEditor } from "./components/providers-editor";
import { ReadPanel } from "./components/read-panel";
import { AboutPanel } from "./components/about-panel";
import { UpdateBanner } from "./components/update-banner";
import {
  InfoIcon,
  KeyIcon,
  LogIcon,
  MicIcon,
  ModesIcon,
  PlugIcon,
  SpeakerIcon,
  WaveMark,
} from "./components/icons";

const SECTIONS = [
  { id: "dictate", key: "dictate", Icon: MicIcon, dividerBefore: false },
  { id: "modes", key: "modes", Icon: ModesIcon, dividerBefore: false },
  { id: "providers", key: "providers", Icon: PlugIcon, dividerBefore: false },
  { id: "behaviour", key: "hotkeys", Icon: KeyIcon, dividerBefore: false },
  { id: "history", key: "history", Icon: LogIcon, dividerBefore: false },
  // Everything from here down is a second group — dictation is the app's main job,
  // this is the "by the way" row. See ui-spec §1.
  { id: "read", key: "read", Icon: SpeakerIcon, dividerBefore: true },
  { id: "about", key: "about", Icon: InfoIcon, dividerBefore: true },
] as const;

type SectionId = (typeof SECTIONS)[number]["id"];

const SAVE_DEBOUNCE_MS = 400;

/** Backoff for the settings load: 200ms, 400ms, 800ms, 1.6s, 3.2s, then give up and say so. */
const LOAD_RETRIES = 5;
const LOAD_RETRY_MS = 200;

export function SettingsApp() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [section, setSection] = useState<SectionId>("dictate");
  const [saveError, setSaveError] = useState<string | null>(null);
  const [presets, setPresets] = useState<LlmPreset[]>([]);
  const [version, setVersion] = useState<string | null>(null);
  const [bannerDismissed, setBannerDismissed] = useState(false);
  const { phase } = useDictationEvents();
  const t = useT();
  // Never checks (and never shows a banner) until onboarding is done — a first-run
  // user should not see "a new version is out" mid-walkthrough.
  const updateCheck = useUpdateCheck(
    Boolean(settings?.auto_check_updates) && Boolean(settings?.onboarding_completed),
  );

  const saveTimer = useRef<number | undefined>(undefined);
  // Distinguishes local edits from settings pushed by the backend (e.g. a mode switched
  // by hotkey), so an inbound update is not echoed straight back.
  const dirty = useRef(false);

  // This one call gates the entire window — every other tab is behind `settings` being
  // non-null. Swallowing its rejection therefore cost the whole session: the app sat on
  // "Loading" forever and said nothing about why. Retry a few times (the failures worth
  // recovering from are transient), then show the error and offer the retry by hand.
  const loadSettings = useCallback(async () => {
    setLoadError(null);
    for (let attempt = 0; ; attempt++) {
      try {
        setSettings(await api.getSettings());
        return;
      } catch (e) {
        if (attempt >= LOAD_RETRIES) {
          setLoadError(String(e));
          return;
        }
        await new Promise((r) => window.setTimeout(r, LOAD_RETRY_MS << attempt));
      }
    }
  }, []);

  useEffect(() => {
    getVersion().then((v) => {
      setVersion(v);
      // The rail already shows the version in small type; the title bar is the
      // spot people actually glance at, so it carries the version too.
      getCurrentWindow().setTitle(`Tocky Voice v${v}`).catch(() => undefined);
    }).catch(() => undefined);
    void loadSettings();
    api.listLlmPresets().then(setPresets).catch(() => undefined);
    const off = listen(api.EVENTS.settingsChanged, () => {
      if (dirty.current) return;
      api.getSettings().then(setSettings).catch(() => undefined);
    });
    return () => {
      off.then((fn) => fn()).catch(() => undefined);
    };
  }, [loadSettings]);

  const update = useCallback((next: AppSettings) => {
    dirty.current = true;
    setSettings(next);
    window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => {
      api
        .saveSettings(next)
        .then(() => setSaveError(null))
        .catch((e) => setSaveError(String(e)))
        .finally(() => {
          dirty.current = false;
        });
    }, SAVE_DEBOUNCE_MS);
  }, []);

  if (!settings) {
    return (
      <div className="loading">
        {loadError === null ? (
          t.nav.loading
        ) : (
          <div className="loading__failure">
            <p className="loading__headline">{t.nav.loadFailed}</p>
            <pre className="loading__detail">{loadError}</pre>
            <button className="btn-primary" type="button" onClick={() => void loadSettings()}>
              {t.nav.retry}
            </button>
          </div>
        )}
      </div>
    );
  }

  // Shown over everything: on a fresh install none of the tabs behind it can do
  // anything useful until the walkthrough's steps are done anyway.
  if (!settings.onboarding_completed) {
    return (
      <OnboardingFlow
        settings={settings}
        onSettingsChange={update}
        onDone={() => setSection("dictate")}
      />
    );
  }

  const pip =
    phase === "recording" ? "live" : phase === "idle" ? "idle" : "busy";

  return (
    <div className="app">
      <nav className="rail">
        <div className="rail__brand">
          <WaveMark className="rail__mark" />
          <div>
            <div className="rail__name">Tocky Voice</div>
            <span className="rail__sub">{version ? `v${version}` : ""}</span>
          </div>
        </div>

        {SECTIONS.map(({ id, key, Icon, dividerBefore }) => (
          <div key={id}>
            {dividerBefore && <hr className="rail__divider" />}
            <button
              className={`rail__item ${section === id ? "rail__item--on" : ""}`}
              onClick={() => setSection(id)}
            >
              <Icon className="rail__icon" />
              {t.nav[key]}
            </button>
          </div>
        ))}

        <div className="rail__spacer" />

        <div className="rail__status">
          <span className={`rail__pip rail__pip--${pip}`} />
          {phase === "idle" ? t.nav.idle : t.phase[phase]}
        </div>
      </nav>

      <main className="view">
        <div className="view__inner">
          {saveError && <div className="notice notice--error">{saveError}</div>}
          {updateCheck.state === "available" && !bannerDismissed && updateCheck.update && (
            <UpdateBanner
              version={updateCheck.update.version}
              onSeeWhatsNew={() => setSection("about")}
              onDismiss={() => setBannerDismissed(true)}
            />
          )}

          {section === "dictate" && (
            <DictationPanel settings={settings} onSettingsChange={update} />
          )}
          {section === "modes" && (
            <ModesEditor settings={settings} onSettingsChange={update} />
          )}
          {section === "providers" && (
            <ProvidersEditor settings={settings} onSettingsChange={update} />
          )}
          {section === "behaviour" && (
            <BehaviourEditor settings={settings} onSettingsChange={update} />
          )}
          {section === "history" && <HistoryList />}
          {section === "read" && (
            <ReadPanel settings={settings} onSettingsChange={update} />
          )}
          {section === "about" && (
            <AboutPanel presets={presets} version={version} update={updateCheck} />
          )}
        </div>
      </main>
    </div>
  );
}
