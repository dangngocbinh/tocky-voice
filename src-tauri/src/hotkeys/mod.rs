//! Global hotkey binding. Every binding is a regular accelerator handled by the
//! global-shortcut plugin, and every one of them acts on key-down.

use crate::read;
use crate::session;
use crate::settings::AppSettings;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HotkeyAction {
    /// Press once to start, again to stop and transcribe.
    Toggle,
    /// Discard the current take.
    Cancel,
    NextMode,
    /// Switch to a mode and immediately start recording in it.
    SelectMode(String),
    /// Reads the current selection aloud in the active read mode; a second press stops.
    Read,
    /// Flow C: capture the selection, then take a spoken instruction for it.
    ReadWithVoice,
    /// Switch read mode and start reading immediately.
    SelectReadMode(String),
}

#[derive(Default)]
pub struct HotkeyRegistry {
    bindings: Mutex<HashMap<Shortcut, HotkeyAction>>,
}

impl HotkeyRegistry {
    pub fn action_for(&self, shortcut: &Shortcut) -> Option<HotkeyAction> {
        self.bindings.lock().ok()?.get(shortcut).cloned()
    }
}

/// Releases every binding. Used while the settings UI is recording a new shortcut —
/// otherwise pressing the combination you want to assign would fire the action instead
/// of being captured.
pub fn suspend(app: &AppHandle) {
    let registry = app.state::<HotkeyRegistry>();
    let _ = app.global_shortcut().unregister_all();
    if let Ok(mut bindings) = registry.bindings.lock() {
        bindings.clear();
    }
    log::debug!("hotkeys suspended for recording");
}

/// The bindings `settings` calls for, pulled out of [`apply`] so the rule "nothing
/// read-related is registered while the feature is off" is testable without a real
/// `AppHandle`.
fn wanted_bindings(settings: &AppSettings) -> Vec<(String, HotkeyAction)> {
    let mut wanted: Vec<(String, HotkeyAction)> = Vec::new();
    if let Some(acc) = settings.hotkeys.toggle.clone() {
        wanted.push((acc, HotkeyAction::Toggle));
    }
    if let Some(acc) = settings.hotkeys.cancel.clone() {
        wanted.push((acc, HotkeyAction::Cancel));
    }
    if let Some(acc) = settings.hotkeys.next_mode.clone() {
        wanted.push((acc, HotkeyAction::NextMode));
    }
    for mode in &settings.modes {
        if let Some(acc) = mode.hotkey.clone() {
            wanted.push((acc, HotkeyAction::SelectMode(mode.id.clone())));
        }
    }

    // The whole point of the feature defaulting to off: no binding, no tray entry, no
    // window, nothing registered — someone who never turns this on cannot tell it
    // exists beyond one rail item. See plan.md's "Bất biến phải giữ" #1.
    if settings.tts.enabled {
        if let Some(acc) = settings.hotkeys.read.clone() {
            wanted.push((acc, HotkeyAction::Read));
        }
        if let Some(acc) = settings.hotkeys.read_with_voice.clone() {
            wanted.push((acc, HotkeyAction::ReadWithVoice));
        }
        for mode in &settings.read_modes {
            if let Some(acc) = mode.hotkey.clone() {
                wanted.push((acc, HotkeyAction::SelectReadMode(mode.id.clone())));
            }
        }
    }
    wanted
}

/// Rebinds every hotkey to match `settings`. Safe to call repeatedly — existing
/// bindings are torn down first, so saving settings re-applies them immediately.
pub fn apply(app: &AppHandle, settings: &AppSettings) {
    suspend(app);
    let registry = app.state::<HotkeyRegistry>();

    for (accelerator, action) in wanted_bindings(settings) {
        let Ok(shortcut) = Shortcut::from_str(&accelerator) else {
            log::warn!("ignoring unparseable hotkey {accelerator:?}");
            continue;
        };
        if let Err(e) = app.global_shortcut().register(shortcut) {
            log::warn!(
                "could not register hotkey {accelerator:?} ({action:?}) — another app \
                 probably owns it: {e}"
            );
            continue;
        }
        log::info!("hotkey registered: {accelerator} -> {action:?}");
        if let Ok(mut bindings) = registry.bindings.lock() {
            bindings.insert(shortcut, action);
        }
    }
}

/// Global-shortcut plugin callback.
pub fn on_shortcut(app: &AppHandle, shortcut: &Shortcut, state: ShortcutState) {
    let Some(action) = app.state::<HotkeyRegistry>().action_for(shortcut) else {
        // Logged rather than ignored: "the hotkey does nothing" is the single most
        // common report, and this line is what separates "the key never reached us"
        // from "it reached us and the action misfired".
        log::debug!("unbound shortcut fired: {shortcut:?} ({state:?})");
        return;
    };
    log::debug!("hotkey {action:?} {state:?}");
    match (action, state) {
        // Every binding fires once, on key-down. Key-up is what a held key sends on
        // release, and acting on it too would run the action twice per press.
        (_, ShortcutState::Released) => {}
        (HotkeyAction::Toggle, _) => session::toggle(app),
        (HotkeyAction::Cancel, _) => session::cancel(app),
        (HotkeyAction::NextMode, _) => session::next_mode(app),
        (HotkeyAction::SelectMode(mode_id), _) => session::start(app, Some(mode_id)),
        (HotkeyAction::Read, _) => read::toggle(app, None),
        (HotkeyAction::ReadWithVoice, _) => read::start_voice_command(app),
        (HotkeyAction::SelectReadMode(mode_id), _) => read::start(app, Some(mode_id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::defaults;

    /// Every factory binding has to survive `Shortcut::from_str`, or `apply` skips it
    /// with a log line nobody reads and the app ships with a hotkey that never fires.
    #[test]
    fn every_default_accelerator_parses() {
        let hotkeys = defaults::default_hotkeys();
        let accelerators: Vec<String> = [
            &hotkeys.toggle,
            &hotkeys.cancel,
            &hotkeys.next_mode,
            &hotkeys.read,
            &hotkeys.read_with_voice,
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect();
        assert!(!accelerators.is_empty());
        for accelerator in accelerators {
            assert!(
                Shortcut::from_str(&accelerator).is_ok(),
                "default hotkey {accelerator:?} does not parse"
            );
        }
    }

    /// Dictation has exactly one binding now, so a factory default that is missing
    /// leaves a fresh install with no way to start a take except the tray menu.
    #[test]
    fn a_fresh_install_has_a_dictation_hotkey() {
        assert!(defaults::default_hotkeys().toggle.is_some());
    }

    /// The core invariant of the whole feature: turning it off must leave no trace in
    /// the hotkey registry, not even the two read-aloud bindings a fresh install ships
    /// with — see plan.md's "Bất biến phải giữ" #1.
    #[test]
    fn nothing_read_related_is_registered_while_the_feature_is_off() {
        let mut settings = defaults::default_settings();
        settings.tts.enabled = false;
        assert!(defaults::default_hotkeys().read.is_some());

        let actions: Vec<HotkeyAction> =
            wanted_bindings(&settings).into_iter().map(|(_, a)| a).collect();
        assert!(!actions.contains(&HotkeyAction::Read));
        assert!(!actions.contains(&HotkeyAction::ReadWithVoice));
        assert!(!actions
            .iter()
            .any(|a| matches!(a, HotkeyAction::SelectReadMode(_))));
    }

    #[test]
    fn read_bindings_appear_once_the_feature_is_on() {
        let mut settings = defaults::default_settings();
        settings.tts.enabled = true;

        let actions: Vec<HotkeyAction> =
            wanted_bindings(&settings).into_iter().map(|(_, a)| a).collect();
        assert!(actions.contains(&HotkeyAction::Read));
        assert!(actions.contains(&HotkeyAction::ReadWithVoice));
    }
}
