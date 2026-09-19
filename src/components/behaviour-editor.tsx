/** Hotkeys, sound cues, history retention, and launch-at-login. */

import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import * as api from "../lib/api";
import type { AppSettings, BindingCheck, HotkeyDelivery, ProbeHit } from "../lib/types";
import { useT } from "../lib/i18n";
import { HotkeyRecorder } from "./hotkey-recorder";
import { formatAccelerator } from "../lib/format-accelerator";
import { Switch } from "./providers-editor";

interface Props {
  settings: AppSettings;
  onSettingsChange: (settings: AppSettings) => void;
}

export function BehaviourEditor({ settings, onSettingsChange }: Props) {
  const t = useT();
  const [delivery, setDelivery] = useState<HotkeyDelivery | null>(null);

  // Re-read after every save: recording a shortcut re-applies bindings on the Rust
  // side, and on a manual Wayland desktop the command lines below embed the
  // accelerator, so they'd otherwise go stale the moment one changed.
  useEffect(() => {
    api.hotkeyDelivery().then(setDelivery).catch(() => setDelivery(null));
  }, [settings.hotkeys, settings.modes]);

  const patchHotkeys = (patch: Partial<AppSettings["hotkeys"]>) =>
    onSettingsChange({ ...settings, hotkeys: { ...settings.hotkeys, ...patch } });

  const patchAudio = (patch: Partial<AppSettings["audio"]>) =>
    onSettingsChange({ ...settings, audio: { ...settings.audio, ...patch } });

  const patchHistory = (patch: Partial<AppSettings["history"]>) =>
    onSettingsChange({ ...settings, history: { ...settings.history, ...patch } });

  return (
    <>
      <h1 className="view__title">{t.behaviour.title}</h1>
      <p className="view__lede">
{t.behaviour.lede}
      </p>

      <section className="section">
        <h2 className="section__title">{t.behaviour.generalSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.uiLanguage}</div>
            <span className="row__hint">{t.behaviour.uiLanguageHint}</span>
          </div>
          <div className="row__control">
            <select
              value={settings.ui_language}
              onChange={(e) =>
                onSettingsChange({
                  ...settings,
                  ui_language: e.target.value as AppSettings["ui_language"],
                })
              }
            >
              <option value="system">{t.behaviour.followSystem}</option>
              <option value="en">English</option>
              <option value="vi">Tiếng Việt</option>
            </select>
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.theme}</div>
            <span className="row__hint">{t.behaviour.themeHint}</span>
          </div>
          <div className="row__control">
            <select
              value={settings.theme}
              onChange={(e) =>
                onSettingsChange({
                  ...settings,
                  theme: e.target.value as AppSettings["theme"],
                })
              }
            >
              <option value="system">{t.behaviour.followSystem}</option>
              <option value="light">{t.behaviour.themeLight}</option>
              <option value="dark">{t.behaviour.themeDark}</option>
            </select>
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.onboarding.rerunTitle}</div>
            <span className="row__hint">{t.onboarding.rerunBody}</span>
          </div>
          <div className="row__control">
            <button
              onClick={() => onSettingsChange({ ...settings, onboarding_completed: false })}
            >
              {t.onboarding.rerun}
            </button>
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.update.autoCheckLabel}</div>
            <span className="row__hint">{t.update.autoCheckHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.auto_check_updates}
              onChange={(auto_check_updates) =>
                onSettingsChange({ ...settings, auto_check_updates })
              }
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.shortcutsSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.toggle}</div>
            <span className="row__hint">{t.behaviour.toggleHint}</span>
          </div>
          <div className="row__control">
            <HotkeyRecorder
              value={settings.hotkeys.toggle}
              onChange={(toggle) => patchHotkeys({ toggle })}
            />
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.cancel}</div>
            <span className="row__hint">{t.behaviour.cancelHint}</span>
          </div>
          <div className="row__control">
            <HotkeyRecorder
              value={settings.hotkeys.cancel}
              onChange={(cancel) => patchHotkeys({ cancel })}
            />
          </div>
        </div>

        <div className="row">
          <div className="row__label">{t.behaviour.nextMode}</div>
          <div className="row__control">
            <HotkeyRecorder
              value={settings.hotkeys.next_mode}
              onChange={(next_mode) => patchHotkeys({ next_mode })}
            />
          </div>
        </div>

        {delivery?.mechanism === "gnome" && (
          <p className="row__hint">{t.behaviour.deliveryGnome}</p>
        )}
        {delivery && <ConflictList checks={delivery.checks} />}
        {delivery && <HotkeyProbe checks={delivery.checks} />}
        {delivery && delivery.mechanism !== "in_process" && delivery.binary && (
          <p className="row__hint mono">
            {t.behaviour.deliveryBinary}: {delivery.binary}
          </p>
        )}
        {delivery?.mechanism === "manual" && delivery.commands.length > 0 && (
          <>
            <p className="row__hint">{t.behaviour.deliveryManual}</p>
            {delivery.commands.map((c) => (
              <ManualCommandRow key={c.command} command={c} />
            ))}
          </>
        )}
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.feedbackSection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.sounds}</div>
<span className="row__hint">{t.behaviour.soundsHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.audio.feedback_sounds}
              onChange={(feedback_sounds) => patchAudio({ feedback_sounds })}
            />
          </div>
        </div>

        <div className="row">
          <div className="row__label">
            {t.behaviour.volume}
            <span className="row__hint mono">
              {Math.round(settings.audio.feedback_volume * 100)}%
            </span>
          </div>
          <div className="row__control">
            <input
              type="range"
              min={0}
              max={1}
              step={0.05}
              value={settings.audio.feedback_volume}
              onChange={(e) => patchAudio({ feedback_volume: Number(e.target.value) })}
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.historySection}</h2>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.keepHistory}</div>
            <span className="row__hint">{t.behaviour.keepHistoryHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.history.enabled}
              onChange={(enabled) => patchHistory({ enabled })}
            />
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.keepAudio}</div>
            <span className="row__hint">{t.behaviour.keepAudioHint}</span>
          </div>
          <div className="row__control">
            <Switch
              checked={settings.history.keep_audio}
              onChange={(keep_audio) => patchHistory({ keep_audio })}
            />
          </div>
        </div>

        <div className="row">
          <div className="row__label">{t.behaviour.maxEntries}</div>
          <div className="row__control">
            <input
              type="number"
              min={10}
              max={5000}
              value={settings.history.max_entries}
              onChange={(e) => patchHistory({ max_entries: Number(e.target.value) })}
            />
          </div>
        </div>

        <div className="row">
          <div>
            <div className="row__label">{t.behaviour.retention}</div>
            <span className="row__hint">{t.behaviour.retentionHint}</span>
          </div>
          <div className="row__control">
            <input
              type="number"
              min={0}
              max={365}
              value={settings.history.audio_retention_days}
              onChange={(e) => patchHistory({ audio_retention_days: Number(e.target.value) })}
            />
          </div>
        </div>
      </section>

      <section className="section">
        <h2 className="section__title">{t.behaviour.startupSection}</h2>
        <div className="row">
          <div className="row__label">{t.behaviour.autostart}</div>
          <div className="row__control">
            <Switch
              checked={settings.autostart}
              onChange={(autostart) => onSettingsChange({ ...settings, autostart })}
            />
          </div>
        </div>
      </section>
    </>
  );
}

/** Strips the product prefix the desktop needs but a row inside the app does not. */
function shortLabel(label: string): string {
  return label.replace(/^.*?—\s*/, "");
}

/** Every configured shortcut that will not fire, and what is standing in its way.
 *
 *  The per-row warning in the recorder answers "is this key free?" while you are
 *  choosing one. This answers the question the bug report actually asked — "why does
 *  nothing happen when I press it?" — for shortcuts that were set long ago and have been
 *  quietly dead ever since, because a desktop shortcut was added on top of them. */
function ConflictList({ checks }: { checks: BindingCheck[] }) {
  const t = useT();
  const broken = checks.filter((c) => c.status !== "ok");
  if (broken.length === 0) return null;

  return (
    <div className="notice notice--warn hk-conflicts">
      <div>
        <div className="row__label">{t.behaviour.conflictsTitle}</div>
        <ul className="hk-conflicts__list">
          {broken.map((c) => (
            <li key={c.action}>
              <kbd>{formatAccelerator(c.accelerator)}</kbd> {shortLabel(c.label)}
              {c.owner
                ? ` — ${c.owner}`
                : c.status === "unsupported"
                  ? ` — ${t.recorder.unsupported}`
                  : ` — ${t.recorder.takenByOther}`}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

/** "Press it and see" — the one thing the settings screen could never answer.
 *
 *  While a probe is on, a hotkey that reaches the app announces which action it was
 *  bound to and runs nothing at all: testing dictation does not start a recording, and
 *  testing read-aloud does not start talking. Silence for the whole window is itself the
 *  answer, and it is the answer that used to arrive as no feedback whatsoever. */
function HotkeyProbe({ checks }: { checks: BindingCheck[] }) {
  const t = useT();
  const [running, setRunning] = useState(false);
  const [hit, setHit] = useState<string | null>(null);
  const [expired, setExpired] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  const stop = useCallback(() => {
    window.clearTimeout(timer.current);
    setRunning(false);
    api.stopHotkeyProbe().catch(() => undefined);
  }, []);

  // A probe left armed swallows every hotkey press, so leaving this screen — or the
  // window being closed on it — has to disarm it. The Rust side expires it too, in case
  // the frontend never gets the chance.
  useEffect(() => stop, [stop]);

  useEffect(() => {
    if (!running) return;
    const off = listen<ProbeHit>(api.EVENTS.hotkeyFired, ({ payload }) => {
      const match = checks.find((c) => c.action === payload.action);
      setHit(match ? shortLabel(match.label) : payload.action);
      stop();
    });
    return () => {
      off.then((fn) => fn()).catch(() => undefined);
    };
  }, [running, checks, stop]);

  const start = () => {
    setHit(null);
    setExpired(false);
    setRunning(true);
    api.startHotkeyProbe().catch(() => undefined);
    timer.current = window.setTimeout(() => {
      setExpired(true);
      stop();
    }, 20_000);
  };

  return (
    <div className="row">
      <div>
        <div className="row__label">{t.behaviour.probeTitle}</div>
        <span className="row__hint">
          {running
            ? t.behaviour.probeWaiting
            : hit
              ? t.behaviour.probeHit.replace("{action}", hit)
              : expired
                ? t.behaviour.probeNothing
                : t.behaviour.probeHint}
        </span>
      </div>
      <div className="row__control">
        <button
          type="button"
          className={running ? "rec__btn rec__btn--live" : ""}
          onClick={() => (running ? stop() : start())}
        >
          {running ? t.behaviour.probeStop : t.behaviour.probeStart}
        </button>
      </div>
    </div>
  );
}

/** One shortcut a manual-Wayland desktop needs the user to bind themselves, with a
 *  copy button so the command line doesn't have to be retyped by hand into a dialog
 *  that will reject the smallest typo. */
function ManualCommandRow({ command }: { command: { label: string; accelerator: string; command: string } }) {
  const t = useT();
  const [copied, setCopied] = useState(false);

  return (
    <div className="row">
      <div>
        <div className="row__label">{command.label}</div>
        <span className="row__hint mono">{command.command}</span>
      </div>
      <div className="row__control">
        <button
          type="button"
          className="btn-quiet"
          onClick={() => {
            api
              .copyText(command.command)
              .then(() => {
                setCopied(true);
                setTimeout(() => setCopied(false), 1500);
              })
              .catch(() => undefined);
          }}
        >
          {copied ? t.behaviour.deliveryManualCopied : t.behaviour.deliveryManualCopy}
        </button>
      </div>
    </div>
  );
}
