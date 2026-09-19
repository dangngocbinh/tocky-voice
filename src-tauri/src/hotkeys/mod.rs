//! Global hotkey binding.
//!
//! Two mechanisms, picked by what the session can actually deliver. Everywhere the app
//! is allowed to grab keys for itself — macOS, Windows, X11 — a binding is a regular
//! accelerator handled by the global-shortcut plugin, acting on key-down. On a Wayland
//! desktop no client may grab a global key at all, so the shortcuts are written into
//! the desktop's own keyboard settings instead and come back to us as a second launch
//! carrying a flag (see [`action_from_args`] and [`gnome`]). Exactly one of the two is
//! ever live: running both would fire every action twice per press.

#[cfg(target_os = "linux")]
pub mod gnome;

use crate::read;
use crate::session;
use crate::settings::AppSettings;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
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

impl HotkeyAction {
    /// The command-line flag that reproduces this action through a second launch.
    ///
    /// This is the other half of [`action_from_args`], and the pair has to stay in step:
    /// a desktop keybinding written from here is parsed back there, so a flag spelled
    /// differently in the two places produces a shortcut that launches the app and does
    /// nothing else.
    pub fn cli_arg(&self) -> String {
        match self {
            Self::Toggle => "--toggle".to_owned(),
            Self::Cancel => "--cancel".to_owned(),
            Self::NextMode => "--next-mode".to_owned(),
            Self::Read => "--read".to_owned(),
            Self::ReadWithVoice => "--read-voice".to_owned(),
            Self::SelectMode(id) => format!("--mode={id}"),
            Self::SelectReadMode(id) => format!("--read-mode={id}"),
        }
    }

    /// A stable, filesystem-and-dconf-safe name for this action.
    ///
    /// Used as the last path segment of the GNOME shortcut this action is installed as,
    /// so it has to be derived from the action rather than from its position in a list:
    /// a user deleting one mode must not silently re-point another mode's shortcut.
    pub fn slug(&self) -> String {
        let raw = self.cli_arg();
        let raw = raw.trim_start_matches('-');
        raw.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect()
    }
}

/// One shortcut, ready to be registered in-process or handed to the desktop.
#[derive(Debug, Clone)]
pub struct Binding {
    pub accelerator: String,
    pub action: HotkeyAction,
    /// How the shortcut names itself in the desktop's keyboard settings. Unused by the
    /// in-process path, where nothing ever displays it.
    pub label: String,
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
    // On Wayland the shortcuts live in the desktop, and the desktop is *very* good at
    // consuming them: a key GNOME holds never reaches our window at all, so recording a
    // combination that is already bound would capture nothing while the old action ran.
    // This is what made re-binding a Super shortcut look like the recorder was broken.
    #[cfg(target_os = "linux")]
    gnome::clear();
    log::debug!("hotkeys suspended for recording");
}

/// The bindings `settings` calls for, pulled out of [`apply`] so the rule "nothing
/// read-related is registered while the feature is off" is testable without a real
/// `AppHandle`.
fn wanted_bindings(settings: &AppSettings) -> Vec<Binding> {
    let vi = prefers_vietnamese(settings);
    let mut wanted: Vec<Binding> = Vec::new();
    let mut push = |accelerator: Option<String>, action: HotkeyAction, label: String| {
        if let Some(accelerator) = accelerator {
            wanted.push(Binding {
                accelerator,
                action,
                label,
            });
        }
    };

    push(
        settings.hotkeys.toggle.clone(),
        HotkeyAction::Toggle,
        label(vi, "Đọc chính tả", "Dictate"),
    );
    push(
        settings.hotkeys.cancel.clone(),
        HotkeyAction::Cancel,
        label(vi, "Huỷ bản ghi", "Cancel recording"),
    );
    push(
        settings.hotkeys.next_mode.clone(),
        HotkeyAction::NextMode,
        label(vi, "Đổi mode", "Next mode"),
    );
    for mode in &settings.modes {
        push(
            mode.hotkey.clone(),
            HotkeyAction::SelectMode(mode.id.clone()),
            label(
                vi,
                &format!("Mode: {}", mode.name),
                &format!("Mode: {}", mode.name),
            ),
        );
    }

    // The whole point of the feature defaulting to off: no binding, no tray entry, no
    // window, nothing registered — someone who never turns this on cannot tell it
    // exists beyond one rail item. See plan.md's "Bất biến phải giữ" #1.
    if settings.tts.enabled {
        push(
            settings.hotkeys.read.clone(),
            HotkeyAction::Read,
            label(vi, "Đọc to", "Read aloud"),
        );
        push(
            settings.hotkeys.read_with_voice.clone(),
            HotkeyAction::ReadWithVoice,
            label(vi, "Đọc to + ra lệnh", "Read aloud with a command"),
        );
        for mode in &settings.read_modes {
            push(
                mode.hotkey.clone(),
                HotkeyAction::SelectReadMode(mode.id.clone()),
                label(
                    vi,
                    &format!("Đọc — {}", mode.name),
                    &format!("Read — {}", mode.name),
                ),
            );
        }
    }
    wanted
}

/// How this session's shortcuts actually reach the app, and — when the answer is "only
/// if you bind them yourself" — the exact command line each one needs.
///
/// The settings screen showed a keyboard shortcut and nothing else, which on Wayland
/// was a claim the app could not keep: the combination on screen was stored but never
/// delivered, while whatever the user had typed into their desktop's own dialog was
/// what actually worked. Reporting the mechanism is what lets that screen stop lying.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Delivery {
    pub mechanism: Mechanism,
    /// Populated only for [`Mechanism::Manual`] — one entry per shortcut the user has
    /// to bind in their own desktop, already carrying this install's binary path.
    pub commands: Vec<DesktopCommand>,
    /// The executable the desktop is pointed at. Worth showing: a shortcut installed by
    /// one build and a running app from another is a real way to get a key that fires
    /// and does nothing, and the path is the only visible difference between them.
    pub binary: String,
    /// Every shortcut currently configured, each with whatever is wrong with it. This is
    /// what lets the settings screen mark a single offending row instead of showing one
    /// vague warning — or nothing at all, as it did while the bug report was written.
    pub checks: Vec<BindingCheck>,
}

/// One configured shortcut and its verdict, ready to render next to its recorder.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BindingCheck {
    /// Matches [`ProbeHit::action`], so a row can light up when its key is tested.
    pub action: String,
    pub label: String,
    pub accelerator: String,
    pub status: CheckStatus,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mechanism {
    /// The app grabs the keys itself: macOS, Windows, and X11 sessions.
    InProcess,
    /// Written into GNOME's keyboard settings, because Wayland forbids the grab.
    Gnome,
    /// A Wayland desktop we cannot configure — the user binds the commands by hand.
    Manual,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DesktopCommand {
    pub label: String,
    pub accelerator: String,
    pub command: String,
}

pub fn delivery(settings: &AppSettings) -> Delivery {
    #[cfg(target_os = "linux")]
    {
        if gnome::is_active() {
            return Delivery {
                mechanism: Mechanism::Gnome,
                commands: Vec::new(),
                binary: binary_path(),
                checks: binding_checks(settings),
            };
        }
        if gnome::is_wayland() {
            let commands = wanted_bindings(settings)
                .into_iter()
                .map(|binding| DesktopCommand {
                    label: binding.label,
                    accelerator: binding.accelerator,
                    command: gnome::command_line(&binding.action.cli_arg()),
                })
                .collect();
            return Delivery {
                mechanism: Mechanism::Manual,
                commands,
                binary: binary_path(),
                checks: binding_checks(settings),
            };
        }
    }
    Delivery {
        mechanism: Mechanism::InProcess,
        commands: Vec::new(),
        binary: binary_path(),
        checks: binding_checks(settings),
    }
}

fn binary_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Runs [`check`] over every shortcut the settings currently ask for.
fn binding_checks(settings: &AppSettings) -> Vec<BindingCheck> {
    wanted_bindings(settings)
        .into_iter()
        .map(|binding| {
            let mut verdict = check(settings, &binding.accelerator, Some(&binding.action));
            if verdict.status == CheckStatus::Ok && was_refused(&binding.accelerator) {
                verdict.status = CheckStatus::TakenByDesktop;
                verdict.owner = None;
            }
            BindingCheck {
                action: binding.action.slug(),
                label: binding.label,
                accelerator: binding.accelerator,
                status: verdict.status,
                owner: verdict.owner,
            }
        })
        .collect()
}

/// Shortcut names show up in the desktop's own keyboard settings, a screen this app
/// does not draw and cannot re-render when the interface language changes — so they are
/// written once, in the language the user reads the app in.
fn label(vi: bool, vietnamese: &str, english: &str) -> String {
    format!("Tocky Voice — {}", if vi { vietnamese } else { english })
}

fn prefers_vietnamese(settings: &AppSettings) -> bool {
    match settings.ui_language.as_str() {
        "vi" => true,
        "en" => false,
        // "system" is resolved in the frontend from `navigator.language`, which is not
        // reachable here; the process locale is the same answer by a different route.
        _ => std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .is_ok_and(|l| l.to_ascii_lowercase().starts_with("vi")),
    }
}

/// Rebinds every hotkey to match `settings`. Safe to call repeatedly — existing
/// bindings are torn down first, so saving settings re-applies them immediately.
pub fn apply(app: &AppHandle, settings: &AppSettings) {
    suspend(app);
    if let Ok(mut refused) = REFUSED.lock() {
        refused.clear();
    }
    let wanted = wanted_bindings(settings);

    // Wayland: the desktop owns the keys. Registering them here as well would be
    // harmless only because nothing ever arrives — but on the day a compositor starts
    // honouring grabs, every action would run twice. One mechanism at a time.
    #[cfg(target_os = "linux")]
    if gnome::is_active() {
        let report = gnome::sync(&wanted);
        log::info!(
            "{} shortcut(s) installed into GNOME; {} could not be translated",
            report.installed,
            report.rejected.len()
        );
        return;
    }
    // An X11 or non-GNOME session grabs its own keys below, and `suspend` above has
    // already removed anything a previous Wayland session installed — without that,
    // every press would fire twice.
    let registry = app.state::<HotkeyRegistry>();
    for Binding {
        accelerator, action, ..
    } in wanted
    {
        let Ok(shortcut) = Shortcut::from_str(&accelerator) else {
            log::warn!("ignoring unparseable hotkey {accelerator:?}");
            continue;
        };
        if let Err(e) = app.global_shortcut().register(shortcut) {
            log::warn!(
                "could not register hotkey {accelerator:?} ({action:?}) — another app \
                 probably owns it: {e}"
            );
            // Remembered, not just logged. A grab the system refuses is the Windows and
            // X11 shape of the same complaint Wayland produces: the shortcut is stored,
            // the screen displays it, and the key belongs to somebody else. The settings
            // screen reads this back through `binding_checks`.
            if let Ok(mut refused) = REFUSED.lock() {
                refused.push(accelerator.clone());
            }
            continue;
        }
        log::info!("hotkey registered: {accelerator} -> {action:?}");
        if let Ok(mut bindings) = registry.bindings.lock() {
            bindings.insert(shortcut, action);
        }
    }
}

// ------------------------------------------------------- is this key usable at all

/// What is wrong with an accelerator, if anything.
///
/// "The shortcut is saved and does nothing" was the whole of the Linux bug report, and
/// every one of these variants is a distinct cause of it that used to look identical
/// from the settings screen: nothing was said, and the key simply never worked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Free as far as this machine can tell.
    Ok,
    /// The desktop itself answers to this combination, and on Wayland the desktop wins:
    /// our shortcut would be installed, listed, and never once delivered.
    TakenByDesktop,
    /// Another of this app's own actions already uses it. Harmless to the desktop,
    /// confusing to the user, and on GNOME the second install silently wins.
    TakenByApp,
    /// The combination cannot be expressed as a shortcut on this desktop — a key the
    /// keysym table has no name for, or a bare letter with no modifier.
    Unsupported,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HotkeyCheck {
    pub accelerator: String,
    pub status: CheckStatus,
    /// Who holds it — a GNOME action, another app's shortcut name, or one of ours.
    pub owner: Option<String>,
}

impl HotkeyCheck {
    fn ok(accelerator: &str) -> Self {
        Self {
            accelerator: accelerator.to_owned(),
            status: CheckStatus::Ok,
            owner: None,
        }
    }
}

/// Everything known against one accelerator: whether this desktop can express it, and
/// whether anything already answers to it.
///
/// `skip_action` is the action being re-bound, if any — without it, editing a shortcut
/// to the value it already has would report a conflict with itself.
pub fn check(
    settings: &AppSettings,
    accelerator: &str,
    skip_action: Option<&HotkeyAction>,
) -> HotkeyCheck {
    if Shortcut::from_str(accelerator).is_err() {
        return HotkeyCheck {
            accelerator: accelerator.to_owned(),
            status: CheckStatus::Unsupported,
            owner: None,
        };
    }

    // Our own bindings first: this is the one check that means the same thing on every
    // platform, and the answer names an action the user recognises.
    for binding in wanted_bindings(settings) {
        if Some(&binding.action) == skip_action {
            continue;
        }
        if accelerators_match(&binding.accelerator, accelerator) {
            return HotkeyCheck {
                accelerator: accelerator.to_owned(),
                status: CheckStatus::TakenByApp,
                owner: Some(binding.label),
            };
        }
    }

    #[cfg(target_os = "linux")]
    if gnome::is_active() {
        let Some(gnome_binding) = gnome::to_gnome_binding(accelerator) else {
            return HotkeyCheck {
                accelerator: accelerator.to_owned(),
                status: CheckStatus::Unsupported,
                owner: None,
            };
        };
        if let Some(owner) = gnome::owner_of(&gnome_binding) {
            return HotkeyCheck {
                accelerator: accelerator.to_owned(),
                status: CheckStatus::TakenByDesktop,
                owner: Some(owner),
            };
        }
    }

    HotkeyCheck::ok(accelerator)
}

/// [`check`] as the settings screen needs it: one field is being edited, and `current`
/// is what it held before the user started typing.
///
/// Editing a field to the value it already has is the commonest thing a recorder sees —
/// someone opens it, changes their mind, and presses the same keys again — and it must
/// not come back as "that shortcut is taken".
pub fn check_for_field(
    settings: &AppSettings,
    accelerator: &str,
    current: Option<&str>,
) -> HotkeyCheck {
    let action = current
        .filter(|current| accelerators_match(current, accelerator))
        .and_then(|current| {
            wanted_bindings(settings)
                .into_iter()
                .find(|b| accelerators_match(&b.accelerator, current))
                .map(|b| b.action)
        });
    check(settings, accelerator, action.as_ref())
}

/// Two accelerators naming the same physical combination, whatever order the modifiers
/// were recorded in — `Control+Shift+KeyX` and `Shift+Control+KeyX` are one shortcut.
fn accelerators_match(a: &str, b: &str) -> bool {
    fn parts(s: &str) -> Vec<String> {
        let mut parts: Vec<String> = s
            .split('+')
            .map(|p| p.trim().to_ascii_lowercase())
            // "Command" and "Super" are the same key off macOS, and a settings file can
            // carry either spelling.
            .map(|p| match p.as_str() {
                "command" | "cmd" | "meta" => "super".to_owned(),
                "ctrl" => "control".to_owned(),
                "option" => "alt".to_owned(),
                _ => p,
            })
            .filter(|p| !p.is_empty())
            .collect();
        parts.sort();
        parts
    }
    !a.is_empty() && parts(a) == parts(b)
}

/// Combinations offered when the user cannot think of a free one — filtered against
/// this actual machine, so what comes back is known to be unclaimed here and now.
///
/// The pool leans on Super because that is the modifier desktop applications leave
/// alone: a `Control+Shift` shortcut installed desktop-wide takes that key away from
/// the terminal and the editor, which is how a "working" hotkey turns into a complaint
/// about copy-paste breaking. Function keys come next for the same reason. What the
/// pool deliberately avoids is `Control+Alt`, which a PC layout also sends as AltGr.
const SUGGESTION_POOL: [&str; 20] = [
    "Super+KeyY",
    "Super+KeyJ",
    "Super+KeyG",
    "Super+KeyB",
    "Super+KeyQ",
    "Super+KeyZ",
    "Super+Backslash",
    "Super+Semicolon",
    "Super+F9",
    "Super+F10",
    "Super+F11",
    "Super+F12",
    "Control+Shift+F9",
    "Control+Shift+F10",
    "Control+Shift+F11",
    "Control+Shift+F12",
    "Alt+Shift+KeyD",
    "Alt+Shift+KeyR",
    "Alt+Shift+KeyJ",
    "Alt+Shift+KeyY",
];

/// Up to `limit` combinations that are free on this machine right now.
pub fn suggestions(
    settings: &AppSettings,
    skip_action: Option<&HotkeyAction>,
    limit: usize,
) -> Vec<String> {
    SUGGESTION_POOL
        .iter()
        .filter(|candidate| check(settings, candidate, skip_action).status == CheckStatus::Ok)
        .take(limit)
        .map(|c| (*c).to_owned())
        .collect()
}

// ------------------------------------------------------------------ trying one out

/// Swallows hotkeys instead of acting on them, so the settings screen can prove a key
/// arrives without starting a recording every time the user tests it.
///
/// Answering "did the key reach the app?" is the thing the old screen could not do at
/// all. Suspending the bindings was no use for it: on Wayland that *removes* them from
/// GNOME, so the very thing under test stops existing for the duration of the test.
static PROBE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Accelerators the system refused to hand over on the last [`apply`], on the platforms
/// where the app grabs its own keys. Empty on Wayland, where nothing is grabbed at all.
static REFUSED: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn was_refused(accelerator: &str) -> bool {
    REFUSED
        .lock()
        .is_ok_and(|refused| refused.iter().any(|r| accelerators_match(r, accelerator)))
}

/// Bumped on every change, so a timeout armed by one probe cannot switch off the next.
static PROBE_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// How long a probe stays armed without the UI switching it off. The settings screen
/// ends its own probe, but a closed window or a reloaded frontend never gets to — and a
/// probe left on is a probe that eats every hotkey press for the rest of the session.
const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(90);

pub fn set_probe(on: bool) {
    let generation = PROBE_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    PROBE.store(on, std::sync::atomic::Ordering::Relaxed);
    log::debug!("hotkey probe {}", if on { "armed" } else { "disarmed" });
    if !on {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(PROBE_TIMEOUT);
        // Only disarm the probe this thread armed; a later one owns the flag now.
        if PROBE_GENERATION.load(std::sync::atomic::Ordering::Relaxed) == generation {
            PROBE.store(false, std::sync::atomic::Ordering::Relaxed);
            log::debug!("hotkey probe expired");
        }
    });
}

fn probing() -> bool {
    PROBE.load(std::sync::atomic::Ordering::Relaxed)
}

/// What the settings screen shows when a tested key arrives: which action it was bound
/// to, so pressing the wrong one reads as "that was Cancel" rather than as success.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProbeHit {
    /// Stable action name — `toggle`, `read`, `mode-raw` — matched to a label in the UI.
    pub action: String,
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
    match state {
        // Every binding fires once, on key-down. Key-up is what a held key sends on
        // release, and acting on it too would run the action twice per press.
        ShortcutState::Released => {}
        ShortcutState::Pressed => dispatch(app, action),
    }
}

/// Runs one action, whichever way it was asked for: an in-process global shortcut, or
/// `tockyvoice --toggle` relayed here by the single-instance plugin.
pub fn dispatch(app: &AppHandle, action: HotkeyAction) {
    // A probe is the settings screen asking "does this key reach the app at all?".
    // Answering it by running the action would start a recording on every test press,
    // which is precisely why testing a hotkey used to be something nobody attempted.
    if probing() {
        log::info!("hotkey probe hit: {action:?}");
        let _ = app.emit(
            crate::state::events::HOTKEY_FIRED,
            ProbeHit {
                action: action.slug(),
            },
        );
        return;
    }
    match action {
        HotkeyAction::Toggle => session::toggle(app),
        HotkeyAction::Cancel => session::cancel(app),
        HotkeyAction::NextMode => session::next_mode(app),
        HotkeyAction::SelectMode(mode_id) => session::start(app, Some(mode_id)),
        HotkeyAction::Read => read::toggle(app, None),
        HotkeyAction::ReadWithVoice => read::start_voice_command(app),
        HotkeyAction::SelectReadMode(mode_id) => read::start(app, Some(mode_id)),
    }
}

/// The action a command line asks for, if it asks for one.
///
/// Wayland compositors do not honour X11-style global grabs, so on GNOME — and on
/// every other Wayland session, which is now the default everywhere — the bindings
/// [`apply`] registers succeed and then never fire: the compositor simply never sends
/// us the keys. The way out is the desktop's own keyboard settings, which do work
/// there: the user binds the combination to `tockyvoice --toggle`, the second launch
/// is relayed to the running instance, and this turns the argument back into an action.
///
/// One action per invocation, first flag wins. Anything unrecognised is ignored rather
/// than rejected, so a stray argument raises the window instead of failing silently.
pub fn action_from_args<I>(args: I) -> Option<HotkeyAction>
where
    I: IntoIterator<Item = String>,
{
    // Skipped: argv[0] is the binary's own path, and on Windows it can be a bare
    // `--toggle`-looking thing only if someone renames the exe, which we needn't cover.
    for arg in args.into_iter().skip(1) {
        let (flag, value) = match arg.split_once('=') {
            Some((flag, value)) => (flag.to_owned(), Some(value.to_owned())),
            None => (arg, None),
        };
        return Some(match (flag.as_str(), value) {
            ("--toggle", _) => HotkeyAction::Toggle,
            ("--cancel", _) => HotkeyAction::Cancel,
            ("--next-mode", _) => HotkeyAction::NextMode,
            ("--read", _) => HotkeyAction::Read,
            ("--read-voice", _) => HotkeyAction::ReadWithVoice,
            ("--mode", Some(id)) => HotkeyAction::SelectMode(id),
            ("--read-mode", Some(id)) => HotkeyAction::SelectReadMode(id),
            _ => continue,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::defaults;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_owned()).collect()
    }

    #[test]
    fn each_flag_maps_to_its_action() {
        for (flag, expected) in [
            ("--toggle", HotkeyAction::Toggle),
            ("--cancel", HotkeyAction::Cancel),
            ("--next-mode", HotkeyAction::NextMode),
            ("--read", HotkeyAction::Read),
            ("--read-voice", HotkeyAction::ReadWithVoice),
        ] {
            let parsed = action_from_args(args(&["/usr/bin/tockyvoice", flag]));
            assert_eq!(parsed, Some(expected), "{flag}");
        }
    }

    #[test]
    fn mode_flags_carry_their_mode_id() {
        assert_eq!(
            action_from_args(args(&["tockyvoice", "--mode=raw"])),
            Some(HotkeyAction::SelectMode("raw".into()))
        );
        assert_eq!(
            action_from_args(args(&["tockyvoice", "--read-mode=summary"])),
            Some(HotkeyAction::SelectReadMode("summary".into()))
        );
    }

    /// The binary's own path is argv[0] on every launch, so treating it as a flag would
    /// make an ordinary double-click fire whatever action its name happened to match.
    #[test]
    fn the_program_path_is_never_read_as_a_flag() {
        assert_eq!(action_from_args(args(&["/usr/bin/tockyvoice"])), None);
        assert_eq!(action_from_args(args(&["--toggle"])), None);
    }

    /// A desktop launcher can pass its own arguments; an unknown one must leave the
    /// second launch meaning "show the window", not silently do nothing at all.
    #[test]
    fn unknown_arguments_ask_for_no_action() {
        assert_eq!(action_from_args(args(&["tockyvoice", "--wat", "-x"])), None);
        assert_eq!(action_from_args(args(&["tockyvoice", "--mode"])), None);
    }

    #[test]
    fn the_first_recognised_flag_wins() {
        assert_eq!(
            action_from_args(args(&["tockyvoice", "--cancel", "--toggle"])),
            Some(HotkeyAction::Cancel)
        );
    }

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
            wanted_bindings(&settings).into_iter().map(|b| b.action).collect();
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
            wanted_bindings(&settings).into_iter().map(|b| b.action).collect();
        assert!(actions.contains(&HotkeyAction::Read));
        assert!(actions.contains(&HotkeyAction::ReadWithVoice));
    }

    /// Modifiers are a set, not a sequence: a shortcut recorded as `Control+Shift+KeyX`
    /// and one typed as `Shift+Control+KeyX` are the same key, and treating them as two
    /// lets the app quietly bind one combination to two different actions.
    #[test]
    fn one_combination_is_recognised_however_it_was_spelled() {
        assert!(accelerators_match("Control+Shift+KeyX", "Shift+Control+KeyX"));
        assert!(accelerators_match("Command+Slash", "Super+Slash"));
        assert!(!accelerators_match("Control+KeyX", "Control+Shift+KeyX"));
        assert!(!accelerators_match("", ""));
    }

    /// The conflict the user meets first is with their own other shortcut, and it is the
    /// one case that reads the same on every platform — so it is answered before any
    /// desktop is consulted, and it names the action rather than the key.
    #[test]
    fn a_key_another_action_already_uses_is_reported_against_that_action() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = Some("Control+Shift+KeyG".into());
        settings.hotkeys.cancel = Some("Control+Shift+KeyH".into());

        let verdict = check(&settings, "Shift+Control+KeyG", None);
        assert_eq!(verdict.status, CheckStatus::TakenByApp);
        assert!(verdict.owner.is_some_and(|o| o.contains("chính tả") || o.contains("Dictate")));
    }

    /// Re-saving a shortcut without changing it must not report that it clashes with
    /// itself — the settings screen re-checks on every keystroke of a save.
    #[test]
    fn rebinding_an_action_does_not_conflict_with_its_own_current_key() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = Some("Control+Shift+KeyG".into());

        let verdict = check(&settings, "Control+Shift+KeyG", Some(&HotkeyAction::Toggle));
        assert_ne!(verdict.status, CheckStatus::TakenByApp);
    }

    /// An accelerator the shortcut parser rejects has to come back as unusable, not as
    /// free: saving it leaves a row that displays a key nothing will ever deliver.
    #[test]
    fn an_unparseable_combination_is_never_reported_as_free() {
        let settings = defaults::default_settings();
        assert_eq!(
            check(&settings, "Control+Nonsense", None).status,
            CheckStatus::Unsupported
        );
    }

    /// Suggestions are offered as one click away from being saved, so every one of them
    /// has to be a combination the app could actually bind.
    #[test]
    fn every_suggestion_is_a_bindable_combination() {
        for candidate in SUGGESTION_POOL {
            assert!(
                Shortcut::from_str(candidate).is_ok(),
                "suggestion {candidate:?} does not parse"
            );
            #[cfg(target_os = "linux")]
            assert!(
                gnome::to_gnome_binding(candidate).is_some(),
                "suggestion {candidate:?} cannot be installed into GNOME"
            );
        }
    }

    /// Whatever else the machine reports, a suggestion that is already one of the user's
    /// own shortcuts is not a suggestion.
    #[test]
    fn suggestions_leave_out_the_keys_already_in_use() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = Some(SUGGESTION_POOL[0].to_owned());

        let offered = suggestions(&settings, None, 5);
        assert!(!offered.contains(&SUGGESTION_POOL[0].to_owned()));
    }

    /// The pool exists because `Control+Alt` is what a PC layout sends as AltGr — a
    /// suggestion built on it behaves differently per keyboard layout.
    #[test]
    fn no_suggestion_leans_on_control_alt() {
        for candidate in SUGGESTION_POOL {
            let lower = candidate.to_ascii_lowercase();
            assert!(
                !(lower.contains("control") && lower.contains("alt")),
                "{candidate} uses Control+Alt"
            );
        }
    }

    /// Opening a recorder and pressing the same keys again is the commonest thing it
    /// sees, and reporting that as a clash with itself is what made the old screen feel
    /// like it was refusing perfectly good shortcuts.
    #[test]
    fn re_recording_a_field_with_its_own_key_is_not_a_clash() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = Some("Control+Shift+KeyG".into());

        let verdict = check_for_field(&settings, "Control+Shift+KeyG", Some("Control+Shift+KeyG"));
        assert_ne!(verdict.status, CheckStatus::TakenByApp);
    }

    /// The exemption is for the field's own key only — moving one action onto another
    /// action's shortcut still has to be reported.
    #[test]
    fn a_field_may_not_borrow_another_actions_key_unreported() {
        let mut settings = defaults::default_settings();
        settings.hotkeys.toggle = Some("Control+Shift+KeyG".into());
        settings.hotkeys.cancel = Some("Control+Shift+KeyH".into());

        let verdict = check_for_field(&settings, "Control+Shift+KeyG", Some("Control+Shift+KeyH"));
        assert_eq!(verdict.status, CheckStatus::TakenByApp);
    }
}
