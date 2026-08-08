//! Global hotkey binding. Every binding is a regular accelerator handled by the
//! global-shortcut plugin, and every one of them acts on key-down.

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

/// Rebinds every hotkey to match `settings`. Safe to call repeatedly — existing
/// bindings are torn down first, so saving settings re-applies them immediately.
pub fn apply(app: &AppHandle, settings: &AppSettings) {
    suspend(app);
    let registry = app.state::<HotkeyRegistry>();

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

    for (accelerator, action) in wanted {
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
        let accelerators: Vec<String> = [&hotkeys.toggle, &hotkeys.cancel, &hotkeys.next_mode]
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
}
