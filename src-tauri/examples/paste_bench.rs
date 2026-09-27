//! Measures whether a portal paste actually lands, and under what conditions it does not.
//!
//! "Sometimes the text is pasted, sometimes nothing arrives" cannot be settled by
//! reading the code: the portal reports success either way, because from its side the
//! keystroke *was* delivered — what differs is the modifier state the compositor ORs
//! into it. This drives the same sequence `inject::portal` does, against a focused text
//! entry that records what it received, and varies one condition at a time.
//!
//! Run the target first, then this:
//!   python3 paste_target.py /tmp/paste_target.txt &
//!   cargo run --example paste_bench -- /tmp/paste_target.txt

use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop};
use ashpd::desktop::{PersistMode, Session};
use std::time::Duration;

const XK_CONTROL_L: i32 = 0xffe3;
const XK_SUPER_L: i32 = 0xffeb;
const XK_V: i32 = 'v' as i32;

/// The delay `inject::paste` waits between writing the clipboard and the keystroke.
const CLIPBOARD_SETTLE: Duration = Duration::from_millis(90);

struct Target {
    path: String,
}

impl Target {
    fn clear(&self) {
        let _ = std::fs::write(format!("{}.clear", self.path), "1");
        std::thread::sleep(Duration::from_millis(150));
    }
    fn read(&self) -> String {
        std::fs::read_to_string(&self.path).unwrap_or_default()
    }
}

async fn press(proxy: &RemoteDesktop<'_>, session: &Session<'_, RemoteDesktop<'_>>, key: i32, state: KeyState) {
    if let Err(e) = proxy.notify_keyboard_keysym(session, key, state).await {
        eprintln!("  notify failed: {e}");
    }
}

/// What a trial varies. Each field is one thing `inject::paste` does, so a column that
/// drops to zero names the line of the real paste path that is at fault.
#[derive(Clone, Copy)]
struct Conditions {
    /// A modifier still down when the keystroke fires.
    hold_super: bool,
    /// The wait between writing the clipboard and pressing Ctrl+V.
    settle: Duration,
    /// Put the previous clipboard contents back this long after the keystroke, as
    /// `inject::paste` does on a detached thread. The target reads the clipboard lazily,
    /// so a restore that beats the read hands it the *old* text — or nothing.
    restore_after: Option<Duration>,
}

/// One trial: clear the target, put `text` on the clipboard, send Ctrl+V, read back.
async fn trial(
    proxy: &RemoteDesktop<'_>,
    session: &Session<'_, RemoteDesktop<'_>>,
    target: &Target,
    text: &str,
    conditions: Conditions,
) -> bool {
    let Conditions {
        hold_super,
        settle,
        restore_after,
    } = conditions;
    target.clear();

    let mut clipboard = arboard::Clipboard::new().expect("clipboard");
    let previous = clipboard.get_text().unwrap_or_default();
    clipboard.set_text(text.to_owned()).expect("set clipboard");
    std::thread::sleep(settle);

    // The condition under test: a modifier still physically down when the paste fires.
    // A hotkey fires on key-*down*, so whether the user has let go by the time the
    // dictation finishes is a matter of how fast the transcription came back.
    if hold_super {
        press(proxy, session, XK_SUPER_L, KeyState::Pressed).await;
        std::thread::sleep(Duration::from_millis(20));
    }

    press(proxy, session, XK_CONTROL_L, KeyState::Pressed).await;
    press(proxy, session, XK_V, KeyState::Pressed).await;
    press(proxy, session, XK_V, KeyState::Released).await;
    press(proxy, session, XK_CONTROL_L, KeyState::Released).await;

    if hold_super {
        press(proxy, session, XK_SUPER_L, KeyState::Released).await;
    }

    if let Some(delay) = restore_after {
        std::thread::sleep(delay);
        let _ = clipboard.set_text(previous);
    }

    std::thread::sleep(Duration::from_millis(400));
    let got = target.read();
    got.contains(text)
}

#[tokio::main]
async fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/paste_target.txt".to_owned());
    let target = Target { path };

    let proxy = RemoteDesktop::new().await.expect("portal");
    let session = proxy.create_session().await.expect("session");

    let token_path = dirs_config().join("portal-restore-token");
    let token = std::fs::read_to_string(&token_path)
        .ok()
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty());
    println!(
        "restore token: {}",
        if token.is_some() { "present" } else { "absent" }
    );

    proxy
        .select_devices(
            &session,
            DeviceType::Keyboard.into(),
            token.as_deref(),
            PersistMode::ExplicitlyRevoked,
        )
        .await
        .expect("select_devices");
    let response = proxy
        .start(&session, None)
        .await
        .expect("start")
        .response()
        .expect("declined");
    if let Some(fresh) = response.restore_token() {
        write_token_0600(&token_path, fresh);
    }
    println!("portal session up\n");

    // Sanity check before any measurement: if a plain letter does not reach the target,
    // the bench is measuring its own focus problem rather than the paste.
    target.clear();
    for key in ['s' as i32, 'a' as i32, 'n' as i32] {
        press(&proxy, &session, key, KeyState::Pressed).await;
        press(&proxy, &session, key, KeyState::Released).await;
        std::thread::sleep(Duration::from_millis(60));
    }
    std::thread::sleep(Duration::from_millis(250));
    let typed = target.read();
    println!("sanity: plain typing reached the target as {typed:?}");
    if !typed.contains("san") {
        println!("\nThe target window is not receiving keystrokes at all — it is not\nfocused. Every number below would be a zero for that reason alone.");
        return;
    }

    let runs = 15;
    let base = Conditions {
        hold_super: false,
        settle: CLIPBOARD_SETTLE,
        restore_after: None,
    };
    let cases: [(&str, Conditions); 6] = [
        ("baseline: no restore at all", base),
        (
            "SUPER held while pasting",
            Conditions {
                hold_super: true,
                ..base
            },
        ),
        (
            "restore after 400ms  <-- what the app does today",
            Conditions {
                restore_after: Some(Duration::from_millis(400)),
                ..base
            },
        ),
        (
            "restore after 150ms",
            Conditions {
                restore_after: Some(Duration::from_millis(150)),
                ..base
            },
        ),
        (
            "restore after 50ms",
            Conditions {
                restore_after: Some(Duration::from_millis(50)),
                ..base
            },
        ),
        (
            "restore after 1500ms",
            Conditions {
                restore_after: Some(Duration::from_millis(1500)),
                ..base
            },
        ),
    ];

    for (label, conditions) in cases {
        let mut ok = 0;
        for i in 0..runs {
            let text = format!("PASTE{i:02}");
            if trial(&proxy, &session, &target, &text, conditions).await {
                ok += 1;
            }
        }
        println!("{ok:>3}/{runs}  {label}");
    }

    // The portal keeps a session object alive until it is told otherwise.
    let _ = session.close().await;
}

/// Writes the restore token the way the app does — 0600.
///
/// The bench shares the app's real token path, so a plain `fs::write` would leave a
/// capability the app deliberately keeps private readable by everyone. (The app's own
/// `private_file` helper is crate-private, hence the duplication here.)
fn write_token_0600(path: &std::path::Path, token: &str) {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    let opened = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path);
    match opened {
        Ok(mut file) => {
            if let Err(e) = file.write_all(token.as_bytes()) {
                eprintln!("could not store the restore token: {e}");
            }
        }
        Err(e) => eprintln!("could not open the token file: {e}"),
    }
}

fn dirs_config() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").expect("HOME"))
        .join(".config/pro.mecode.tockyvoice")
}
