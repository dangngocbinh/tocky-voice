//! Global hotkeys on Wayland, by way of the desktop's own keyboard settings.
//!
//! Wayland has no equivalent of an X11 global grab: a client cannot ask the compositor
//! for a key it does not already have focus for, by design. The global-shortcut plugin
//! therefore *succeeds* on Wayland and then never fires — the registration is real, the
//! key simply never arrives. That combination is the worst possible failure shape,
//! because every log line says the hotkey is bound.
//!
//! What does work is what the desktop binds for itself. GNOME keeps user-defined
//! shortcuts in GSettings, each one a command line it runs on the key; our own
//! single-instance plugin turns a second launch of `tockyvoice --toggle` back into an
//! action (see [`super::action_from_args`]). So on Wayland the app writes its shortcuts
//! into GNOME instead of registering them, and the settings window stays the one place
//! shortcuts are edited — before this, the keys that actually worked were whatever the
//! user had typed into GNOME's own dialog by hand, and the app's settings screen
//! confidently displayed something else entirely.
//!
//! Only GNOME is covered here. KDE, Sway and the rest each keep shortcuts somewhere
//! different, and for those [`super::apply`] falls back to printing the commands the
//! user needs to bind by hand.

use super::Binding;
use std::process::Command;

/// Every key this module writes lives under a path starting with this, which is what
/// makes a sync able to clean up after itself without touching anyone else's shortcuts.
const PATH_PREFIX: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/";
const OUR_PREFIX: &str = "tockyvoice-";
const LIST_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys";
const ITEM_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";

/// What a sync did, so the settings UI can say something more useful than "saved".
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct SyncReport {
    /// Shortcuts now installed in GNOME.
    pub installed: usize,
    /// Accelerators that could not be expressed as a GNOME binding at all.
    pub rejected: Vec<String>,
}

/// True when shortcuts have to go through GNOME because the app cannot grab keys itself.
///
/// Both halves matter. On X11 the global-shortcut plugin works, and installing GNOME
/// bindings *as well* would run every action twice per press — so this deliberately
/// stays false there, and [`clear`] removes anything a previous Wayland session left.
pub fn is_active() -> bool {
    is_wayland() && is_gnome() && has_gsettings()
}

pub fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|t| t.eq_ignore_ascii_case("wayland"))
}

fn is_gnome() -> bool {
    // `XDG_CURRENT_DESKTOP` is colon-separated and distro-flavoured: Ubuntu reports
    // "ubuntu:GNOME", vanilla GNOME reports "GNOME", and a GNOME Classic session reports
    // "GNOME-Classic:GNOME". Matching any segment covers all three.
    std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktops| {
        desktops
            .split(':')
            .any(|d| d.eq_ignore_ascii_case("gnome") || d.eq_ignore_ascii_case("gnome-classic"))
    })
}

fn has_gsettings() -> bool {
    Command::new("gsettings")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Points GNOME at the binary the running app was launched from, rather than at a bare
/// `tockyvoice` that only resolves if the install happens to be on `PATH` — a `.deb`
/// puts it there, an AppImage or a `cargo run` build does not.
pub fn command_line(args: &str) -> String {
    let exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "tockyvoice".to_owned());
    // GNOME parses the command with shell-style quoting, so a path containing a space
    // (an AppImage in "~/My Apps", say) has to be quoted or it arrives as two arguments.
    if exe.contains(|c: char| c.is_whitespace() || c == '\'' || c == '"') {
        format!("'{}' {args}", exe.replace('\'', r"'\''"))
    } else {
        format!("{exe} {args}")
    }
}

/// Installs exactly `bindings` and removes every other shortcut this app has ever
/// written. Shortcuts belonging to anything else are left strictly alone.
pub fn sync(bindings: &[Binding]) -> SyncReport {
    let mut report = SyncReport::default();
    let mut ours: Vec<String> = Vec::new();

    for binding in bindings {
        let Some(gnome_binding) = to_gnome_binding(&binding.accelerator) else {
            log::warn!(
                "cannot express {:?} as a GNOME shortcut — skipped",
                binding.accelerator
            );
            report.rejected.push(binding.accelerator.clone());
            continue;
        };
        let path = format!("{PATH_PREFIX}{OUR_PREFIX}{}/", binding.action.slug());
        let target = format!("{ITEM_SCHEMA}:{path}");

        set(&target, "name", &binding.label);
        set(&target, "command", &command_line(&binding.action.cli_arg()));
        set(&target, "binding", &gnome_binding);

        log::info!(
            "GNOME shortcut installed: {gnome_binding} -> {}",
            binding.action.cli_arg()
        );
        ours.push(path);
        report.installed += 1;
    }

    write_list(ours);
    forget_cached_bindings();
    report
}

/// Removes every shortcut this app installed. Called on X11 and on non-GNOME desktops,
/// so that moving a machine from Wayland to X11 does not leave stale bindings behind
/// firing actions a second time alongside the in-process ones.
pub fn clear() {
    if !is_gnome() || !has_gsettings() {
        return;
    }
    write_list(Vec::new());
    forget_cached_bindings();
}

/// Rewrites the master list so it holds `ours` plus every entry that was not ours,
/// resetting the keys of any of our old paths that just dropped out of it.
fn write_list(ours: Vec<String>) {
    let existing = read_list();
    let foreign: Vec<String> = existing
        .iter()
        .filter(|path| !is_ours(path))
        .cloned()
        .collect();

    // Our own paths that are no longer wanted keep their values in dconf even after the
    // list stops mentioning them, and GNOME's own shortcut dialog then shows a ghost
    // entry with no name. Reset them so nothing is left behind.
    for stale in existing.iter().filter(|p| is_ours(p)) {
        if !ours.contains(stale) {
            reset(&format!("{ITEM_SCHEMA}:{stale}"));
        }
    }

    let list: Vec<String> = foreign.into_iter().chain(ours).collect();
    let value = format!(
        "[{}]",
        list.iter()
            .map(|p| format!("'{p}'"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Err(e) = run(&["set", LIST_SCHEMA, "custom-keybindings", &value]) {
        log::warn!("could not update the GNOME shortcut list: {e}");
    }
}

fn is_ours(path: &str) -> bool {
    path.starts_with(&format!("{PATH_PREFIX}{OUR_PREFIX}"))
}

fn read_list() -> Vec<String> {
    let Ok(raw) = run(&["get", LIST_SCHEMA, "custom-keybindings"]) else {
        return Vec::new();
    };
    parse_path_list(&raw)
}

/// GSettings prints a GVariant array — `@as []` when empty, `['/a/', '/b/']` otherwise.
fn parse_path_list(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches("@as")
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches(['\'', '"']).to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

fn set(target: &str, key: &str, value: &str) {
    if let Err(e) = run(&["set", target, key, value]) {
        log::warn!("could not set {key} on {target}: {e}");
    }
}

fn reset(target: &str) {
    if let Err(e) = run(&["reset-recursively", target]) {
        log::debug!("could not reset {target}: {e}");
    }
}

fn run(args: &[&str]) -> Result<String, String> {
    let out = Command::new("gsettings")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Translates a Tauri accelerator (`Control+Alt+KeyD`) into GNOME's form (`<Control><Alt>d`).
///
/// Returns `None` for anything GNOME cannot bind, rather than guessing: a wrong keysym
/// produces a shortcut that looks installed in the GNOME dialog and answers to no key.
pub fn to_gnome_binding(accelerator: &str) -> Option<String> {
    let mut modifiers = String::new();
    let mut key: Option<String> = None;

    for part in accelerator.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        // A modifier appearing after the key means a malformed accelerator, not a
        // second key — reject rather than silently binding the first half.
        if key.is_some() {
            return None;
        }
        match part.to_ascii_lowercase().as_str() {
            "control" | "ctrl" => modifiers.push_str("<Control>"),
            "alt" | "option" => modifiers.push_str("<Alt>"),
            "shift" => modifiers.push_str("<Shift>"),
            // Tauri spells the Windows/Super key "Command" even off macOS, and a
            // settings file written on a Mac can reach a Linux box through a sync.
            "command" | "cmd" | "super" | "meta" => modifiers.push_str("<Super>"),
            // These mean "Command on macOS, Control elsewhere"; here, elsewhere.
            "commandorcontrol" | "cmdorctrl" => modifiers.push_str("<Control>"),
            _ => key = Some(keysym(part)?),
        }
    }

    let key = key?;
    // A bare letter would be bound as a shortcut that swallows that key everywhere;
    // function keys are the one exception the recorder also allows on their own.
    if modifiers.is_empty() && !is_function_key(&key) {
        return None;
    }
    Some(format!("{modifiers}{key}"))
}

fn is_function_key(keysym: &str) -> bool {
    keysym.len() >= 2
        && keysym.starts_with('F')
        && keysym[1..].chars().all(|c| c.is_ascii_digit())
}

/// Maps a `KeyboardEvent.code` name — what the recorder stores — to an X keysym name.
///
/// X keysyms are what GNOME writes in its own settings, and they agree with neither the
/// DOM names nor the printed key: `Quote` is `apostrophe`, `Backquote` is `grave`, and
/// the letter keys are lowercase while the function keys are not.
fn keysym(code: &str) -> Option<String> {
    if let Some(letter) = code.strip_prefix("Key") {
        // "KeyD" -> "d". Anything longer is not a letter key.
        if letter.len() == 1 && letter.chars().all(|c| c.is_ascii_alphabetic()) {
            return Some(letter.to_ascii_lowercase());
        }
        return None;
    }
    if let Some(digit) = code.strip_prefix("Digit") {
        if digit.len() == 1 && digit.chars().all(|c| c.is_ascii_digit()) {
            return Some(digit.to_owned());
        }
        return None;
    }
    if let Some(pad) = code.strip_prefix("Numpad") {
        return numpad_keysym(pad).map(str::to_owned);
    }
    if is_function_key(code) {
        return Some(code.to_owned());
    }

    let named = match code {
        "Slash" => "slash",
        "Backslash" => "backslash",
        "Period" => "period",
        "Comma" => "comma",
        "Semicolon" => "semicolon",
        "Quote" => "apostrophe",
        "Backquote" => "grave",
        "Minus" => "minus",
        "Equal" => "equal",
        "BracketLeft" => "bracketleft",
        "BracketRight" => "bracketright",
        "Space" => "space",
        "Enter" | "Return" => "Return",
        "Escape" => "Escape",
        "Tab" => "Tab",
        "Backspace" => "BackSpace",
        "Delete" => "Delete",
        "Insert" => "Insert",
        "Home" => "Home",
        "End" => "End",
        "PageUp" => "Page_Up",
        "PageDown" => "Page_Down",
        "ArrowUp" => "Up",
        "ArrowDown" => "Down",
        "ArrowLeft" => "Left",
        "ArrowRight" => "Right",
        // Single printable characters reach here from hand-written settings files.
        other if other.chars().count() == 1 => {
            let c = other.chars().next()?;
            if c.is_ascii_alphanumeric() {
                return Some(c.to_ascii_lowercase().to_string());
            }
            return None;
        }
        _ => return None,
    };
    Some(named.to_owned())
}

fn numpad_keysym(pad: &str) -> Option<&'static str> {
    Some(match pad {
        "0" => "KP_0",
        "1" => "KP_1",
        "2" => "KP_2",
        "3" => "KP_3",
        "4" => "KP_4",
        "5" => "KP_5",
        "6" => "KP_6",
        "7" => "KP_7",
        "8" => "KP_8",
        "9" => "KP_9",
        "Add" => "KP_Add",
        "Subtract" => "KP_Subtract",
        "Multiply" => "KP_Multiply",
        "Divide" => "KP_Divide",
        "Decimal" => "KP_Decimal",
        "Enter" => "KP_Enter",
        _ => return None,
    })
}

// ---------------------------------------------------------------- what is taken

/// Schemas GNOME keeps its own fixed shortcuts in. A combination listed in any of them
/// is consumed by the desktop before any application sees it, which on Wayland includes
/// the custom keybinding we would install — so binding over one of these produces a
/// shortcut that is genuinely installed and genuinely never fires.
const SYSTEM_SCHEMAS: [&str; 5] = [
    "org.gnome.desktop.wm.keybindings",
    "org.gnome.shell.keybindings",
    "org.gnome.settings-daemon.plugins.media-keys",
    "org.gnome.mutter.keybindings",
    "org.gnome.mutter.wayland.keybindings",
];

/// One combination already spoken for on this desktop, and by what.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Taken {
    /// GNOME's own spelling, e.g. `<Super>a`.
    pub binding: String,
    /// Shown to the user: a GNOME action name, or another app's shortcut name.
    pub owner: String,
    /// True when this is one of ours, which a rebind is allowed to replace silently.
    pub ours: bool,
}

/// How long a read of the desktop's shortcuts is reused before going back to gsettings.
///
/// Short on purpose. Every answer costs a dozen `gsettings` processes, and the settings
/// screen asks once per shortcut it draws — without this, opening it fired sixty of them.
/// But the user may well be in GNOME's own keyboard dialog in the other window, so a
/// snapshot that outlived a click over there would be exactly as wrong as no warning.
const CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(2);

static CACHE: std::sync::Mutex<Option<(std::time::Instant, Vec<Taken>)>> =
    std::sync::Mutex::new(None);

/// Every shortcut the desktop currently answers to: GNOME's own, plus every custom
/// keybinding — ours and other applications'.
pub fn taken_bindings() -> Vec<Taken> {
    if let Ok(cache) = CACHE.lock() {
        if let Some((read_at, cached)) = cache.as_ref() {
            if read_at.elapsed() < CACHE_TTL {
                return cached.clone();
            }
        }
    }
    let fresh = read_taken_bindings();
    if let Ok(mut cache) = CACHE.lock() {
        *cache = Some((std::time::Instant::now(), fresh.clone()));
    }
    fresh
}

/// Drops the cached view of the desktop. Called after the app writes its own shortcuts,
/// which is the one moment the cache is known to be stale rather than merely old.
pub fn forget_cached_bindings() {
    if let Ok(mut cache) = CACHE.lock() {
        *cache = None;
    }
}

fn read_taken_bindings() -> Vec<Taken> {
    let mut taken = Vec::new();
    if !has_gsettings() {
        return taken;
    }

    for schema in SYSTEM_SCHEMAS {
        let Ok(dump) = run(&["list-recursively", schema]) else {
            continue;
        };
        for (key, value) in parse_dump(&dump) {
            for binding in parse_binding_value(&value) {
                taken.push(Taken {
                    binding,
                    owner: pretty_key_name(&key),
                    ours: false,
                });
            }
        }
    }

    for path in read_list() {
        let target = format!("{ITEM_SCHEMA}:{path}");
        let Ok(binding) = run(&["get", &target, "binding"]) else {
            continue;
        };
        let binding = binding.trim().trim_matches(['\'', '"']).to_owned();
        if binding.is_empty() {
            continue;
        }
        let owner = run(&["get", &target, "name"])
            .map(|n| n.trim().trim_matches(['\'', '"']).to_owned())
            .unwrap_or_default();
        taken.push(Taken {
            binding,
            owner: if owner.is_empty() {
                path.clone()
            } else {
                owner
            },
            ours: is_ours(&path),
        });
    }
    taken
}

/// Who holds `binding`, if anyone — our own shortcuts excluded, since rebinding one of
/// those is the ordinary case and warning about it would be noise.
pub fn owner_of(binding: &str) -> Option<String> {
    let wanted = normalize(binding)?;
    taken_bindings()
        .into_iter()
        .find(|t| !t.ours && normalize(&t.binding).is_some_and(|n| n == wanted))
        .map(|t| t.owner)
}

/// A comparable form of a GNOME binding: modifiers as an order-independent set, and the
/// key folded to lowercase.
///
/// Both halves are needed. GNOME writes `<Super><Shift>r` where a user might type
/// `<Shift><Super>R`, and comparing those as strings reports "no conflict" for two
/// spellings of one key — the sort of near-miss that makes a warning worthless.
fn normalize(binding: &str) -> Option<(std::collections::BTreeSet<String>, String)> {
    let mut modifiers = std::collections::BTreeSet::new();
    let mut rest = binding.trim();
    while let Some(close) = rest.find('>') {
        if !rest.starts_with('<') {
            break;
        }
        let modifier = rest[1..close].to_ascii_lowercase();
        // GNOME accepts several spellings of the same physical modifier; folding them
        // here keeps `<Primary>x` and `<Control>x` from looking like different keys.
        modifiers.insert(match modifier.as_str() {
            "primary" | "ctrl" => "control".to_owned(),
            "meta" | "mod4" => "super".to_owned(),
            other => other.to_owned(),
        });
        rest = &rest[close + 1..];
    }
    let key = rest.trim();
    if key.is_empty() {
        return None;
    }
    Some((modifiers, key.to_ascii_lowercase()))
}

/// Turns a GSettings key name into something a person can read: `toggle-application-view`
/// becomes "Toggle application view", prefixed so it is clear the desktop owns it.
fn pretty_key_name(key: &str) -> String {
    let words = key.replace(['-', '_'], " ");
    let mut chars = words.chars();
    let titled = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => words,
    };
    format!("GNOME — {titled}")
}

/// Splits `gsettings list-recursively` output into (key, value) pairs.
///
/// Each line is `schema.name key value`, where the value itself contains spaces — so
/// the split is on the first two gaps only, and the schema is dropped.
fn parse_dump(dump: &str) -> Vec<(String, String)> {
    dump.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, ' ');
            let _schema = parts.next()?;
            let key = parts.next()?;
            let value = parts.next()?;
            Some((key.to_owned(), value.to_owned()))
        })
        .collect()
}

/// Reads the bindings out of a GSettings value, which is a list for most keybinding
/// keys (`['<Super>a', '<Alt>b']`) and a bare string for a few of them.
///
/// Values that are neither — a boolean, an integer, the `''` and `@as []` that mean
/// "unbound" — yield nothing, because `list-recursively` dumps every key in the schema
/// and only some of them are shortcuts.
fn parse_binding_value(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.starts_with('[') || value.starts_with("@as") {
        return parse_path_list(value)
            .into_iter()
            .filter(|b| b.contains('<') || is_function_key(b))
            .collect();
    }
    if value.starts_with('\'') {
        let inner = value.trim_matches(['\'', '"']).to_owned();
        if inner.contains('<') || is_function_key(&inner) {
            return vec![inner];
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_and_letters_take_gnome_spelling() {
        assert_eq!(
            to_gnome_binding("Control+Alt+KeyD").as_deref(),
            Some("<Control><Alt>d")
        );
        assert_eq!(
            to_gnome_binding("Control+Shift+KeyX").as_deref(),
            Some("<Control><Shift>x")
        );
    }

    /// The Super key is the whole reason this module exists on a GNOME desktop, and
    /// Tauri spells it "Command" — a settings file that came from a Mac must not end up
    /// binding Control here.
    #[test]
    fn super_is_spelled_super_however_it_arrives() {
        for accelerator in ["Super+Slash", "Command+Slash", "Meta+Slash"] {
            assert_eq!(
                to_gnome_binding(accelerator).as_deref(),
                Some("<Super>slash"),
                "{accelerator}"
            );
        }
    }

    /// These are the names X uses, and none of them is guessable from the DOM name.
    #[test]
    fn punctuation_uses_x_keysym_names() {
        let cases = [
            ("Control+Quote", "<Control>apostrophe"),
            ("Control+Backquote", "<Control>grave"),
            ("Control+Period", "<Control>period"),
            ("Control+BracketLeft", "<Control>bracketleft"),
            ("Control+Enter", "<Control>Return"),
            ("Control+Backspace", "<Control>BackSpace"),
            ("Control+ArrowUp", "<Control>Up"),
            ("Control+PageDown", "<Control>Page_Down"),
            ("Control+Numpad5", "<Control>KP_5"),
        ];
        for (accelerator, expected) in cases {
            assert_eq!(to_gnome_binding(accelerator).as_deref(), Some(expected));
        }
    }

    /// A function key is the one thing worth binding on its own; a bare letter would
    /// take that key away from every application on the desktop.
    #[test]
    fn only_function_keys_bind_without_a_modifier() {
        assert_eq!(to_gnome_binding("F9").as_deref(), Some("F9"));
        assert_eq!(to_gnome_binding("Control+F9").as_deref(), Some("<Control>F9"));
        assert_eq!(to_gnome_binding("KeyD"), None);
        assert_eq!(to_gnome_binding("Slash"), None);
    }

    /// An accelerator we cannot translate has to be reported, not approximated: a
    /// shortcut bound to the wrong key looks installed and answers to nothing.
    #[test]
    fn untranslatable_accelerators_are_rejected() {
        assert_eq!(to_gnome_binding(""), None);
        assert_eq!(to_gnome_binding("Control+Fn"), None);
        assert_eq!(to_gnome_binding("Control+KeyDD"), None);
        assert_eq!(to_gnome_binding("Control+KeyD+Alt"), None);
    }

    #[test]
    fn the_shortcut_list_survives_both_shapes_gsettings_prints() {
        assert!(parse_path_list("@as []").is_empty());
        assert!(parse_path_list("[]").is_empty());
        assert_eq!(
            parse_path_list("['/a/custom0/', '/a/custom1/']"),
            vec!["/a/custom0/", "/a/custom1/"]
        );
    }

    /// Ours are recognised by path prefix alone — that is what lets a sync delete its
    /// own leftovers while leaving every other application's shortcuts untouched.
    #[test]
    fn only_our_own_paths_are_claimed() {
        assert!(is_ours(&format!("{PATH_PREFIX}{OUR_PREFIX}toggle/")));
        assert!(!is_ours(&format!("{PATH_PREFIX}custom0/")));
        assert!(!is_ours("/org/gnome/other/tockyvoice-toggle/"));
    }

    /// A path with a space in it is ordinary on Linux (an AppImage in a user folder),
    /// and an unquoted one silently becomes two arguments — the shortcut then launches
    /// a second copy of the app instead of relaying the action to the running one.
    #[test]
    fn a_command_path_with_spaces_stays_one_argument() {
        let quoted = command_line("--toggle");
        assert!(quoted.ends_with(" --toggle"), "{quoted}");
    }

    /// `<Shift><Super>R` and `<Super><Shift>r` are the same key to the desktop, and a
    /// conflict check that compares the strings would wave the second one through.
    #[test]
    fn modifier_order_and_letter_case_do_not_make_a_new_key() {
        assert_eq!(normalize("<Super><Shift>r"), normalize("<Shift><Super>R"));
        assert_ne!(normalize("<Super>r"), normalize("<Super><Shift>r"));
        assert_eq!(normalize(""), None);
    }

    /// GNOME writes `<Primary>` in some dialogs and `<Control>` in others for the very
    /// same modifier.
    #[test]
    fn the_several_spellings_of_one_modifier_fold_together() {
        assert_eq!(normalize("<Primary>x"), normalize("<Control>x"));
        assert_eq!(normalize("<Meta>x"), normalize("<Super>x"));
    }

    /// `list-recursively` prints `schema key value`, and the value has spaces of its own
    /// — splitting on every gap would truncate every list to its first entry.
    #[test]
    fn a_dump_line_splits_into_key_and_whole_value() {
        let dump = "org.gnome.shell.keybindings toggle-overview ['<Super>s', '<Alt>F1']\n\
                    org.gnome.shell.keybindings nothing @as []";
        assert_eq!(
            parse_dump(dump),
            vec![
                ("toggle-overview".to_owned(), "['<Super>s', '<Alt>F1']".to_owned()),
                ("nothing".to_owned(), "@as []".to_owned()),
            ]
        );
    }

    /// The schema dump carries every key, not just the shortcuts — booleans, numbers and
    /// unbound entries have to yield nothing rather than becoming phantom conflicts.
    #[test]
    fn only_real_bindings_are_read_out_of_a_settings_value() {
        assert_eq!(
            parse_binding_value("['<Super>s', '<Alt>F1']"),
            vec!["<Super>s", "<Alt>F1"]
        );
        assert_eq!(parse_binding_value("'<Control>F9'"), vec!["<Control>F9"]);
        assert_eq!(parse_binding_value("F9"), Vec::<String>::new());
        assert!(parse_binding_value("@as []").is_empty());
        assert!(parse_binding_value("['']").is_empty());
        assert!(parse_binding_value("true").is_empty());
        assert!(parse_binding_value("'disabled'").is_empty());
        assert!(parse_binding_value("120").is_empty());
    }

    /// The owner is shown to someone deciding whether to pick another key, so it has to
    /// read as a sentence fragment, not as a dconf key.
    #[test]
    fn a_settings_key_is_named_in_words() {
        assert_eq!(pretty_key_name("toggle-application-view"), "GNOME — Toggle application view");
    }

    /// Not a unit test but a probe of the machine it runs on: `cargo test -- --ignored
    /// --nocapture what_this_desktop_has_taken` prints what the conflict checker sees,
    /// which is the quickest way to tell a wrong answer from a missing one.
    #[test]
    #[ignore = "reads the live desktop; run by hand when diagnosing"]
    fn what_this_desktop_has_taken() {
        let taken = taken_bindings();
        println!("{} bindings in use on this desktop", taken.len());
        for t in taken.iter().filter(|t| t.ours) {
            println!("  ours: {} -> {}", t.binding, t.owner);
        }
        for probe in ["<Super>a", "<Super>s", "<Super>slash", "<Super>y", "<Control><Shift>F12"] {
            println!("  {probe}: {:?}", owner_of(probe));
        }
    }
}
