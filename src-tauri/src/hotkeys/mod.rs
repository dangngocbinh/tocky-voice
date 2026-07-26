//! Global hotkey binding.
//!
//! Regular accelerators go through the global-shortcut plugin. A held bare modifier
//! (the default push-to-talk) cannot be expressed as an accelerator, so it is handled
//! by a platform listener instead — see [`macos_ptt`].

#[cfg(target_os = "macos")]
pub mod macos_ptt;

use crate::session;
use crate::settings::{AppSettings, PushToTalk};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HotkeyAction {
    /// Press once to start, again to stop and transcribe.
    Toggle,
    /// Discard the current take.
    Cancel,
    NextMode,
    /// Hold to record; release to transcribe.
    PushToTalk,
    /// Switch to a mode and immediately start recording in it.
    SelectMode(String),
}

#[derive(Default)]
pub struct HotkeyRegistry {
    bindings: Mutex<HashMap<Shortcut, HotkeyAction>>,
    /// Clearing this flag stops the modifier-listener thread.
    modifier_listener: Mutex<Option<Arc<AtomicBool>>>,
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
    stop_modifier_listener(&registry);
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

    match &settings.hotkeys.push_to_talk {
        PushToTalk::Disabled => {}
        PushToTalk::Shortcut { accelerator } => {
            wanted.push((accelerator.clone(), HotkeyAction::PushToTalk));
        }
        PushToTalk::Modifier { key } => start_modifier_listener(app, &registry, *key),
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

#[cfg(target_os = "macos")]
fn start_modifier_listener(
    app: &AppHandle,
    registry: &HotkeyRegistry,
    key: crate::settings::ModifierKey,
) {
    if !macos_ptt::has_accessibility_permission() {
        log::warn!(
            "push-to-talk on {key:?} needs Accessibility permission, which has not been \
             granted — the held-modifier listener will not receive key events. Grant it in \
             System Settings → Privacy & Security → Accessibility, then restart the app."
        );
    }

    let app = app.clone();
    let flag = macos_ptt::spawn_listener(key, move |held| {
        log::debug!("push-to-talk modifier {}", if held { "down" } else { "up" });
        if held {
            session::start(&app, None);
        } else {
            session::stop(&app);
        }
    });
    if let Ok(mut slot) = registry.modifier_listener.lock() {
        *slot = Some(flag);
    }
}

#[cfg(not(target_os = "macos"))]
fn start_modifier_listener(
    _app: &AppHandle,
    _registry: &HotkeyRegistry,
    _key: crate::settings::ModifierKey,
) {
    log::warn!(
        "hold-a-modifier push-to-talk is macOS-only for now; \
         choose a regular shortcut for push-to-talk on this platform"
    );
}

fn stop_modifier_listener(registry: &HotkeyRegistry) {
    if let Ok(mut slot) = registry.modifier_listener.lock() {
        if let Some(flag) = slot.take() {
            flag.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Global-shortcut plugin callback.
pub fn on_shortcut(app: &AppHandle, shortcut: &Shortcut, state: ShortcutState) {
    let Some(action) = app.state::<HotkeyRegistry>().action_for(shortcut) else {
        return;
    };
    match (action, state) {
        // Push-to-talk is the only binding that cares about key-up.
        (HotkeyAction::PushToTalk, ShortcutState::Pressed) => session::start(app, None),
        (HotkeyAction::PushToTalk, ShortcutState::Released) => session::stop(app),

        // Everything else fires once, on key-down.
        (_, ShortcutState::Released) => {}
        (HotkeyAction::Toggle, _) => session::toggle(app),
        (HotkeyAction::Cancel, _) => session::cancel(app),
        (HotkeyAction::NextMode, _) => session::next_mode(app),
        (HotkeyAction::SelectMode(mode_id), _) => session::start(app, Some(mode_id)),
    }
}
