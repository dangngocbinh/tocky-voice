//! Synthesizing keystrokes on Wayland, through the desktop's Remote Desktop portal.
//!
//! The XTEST route the rest of this module uses works on Wayland only by way of
//! Xwayland, and since GNOME 45 that comes at a price: Xwayland runs with
//! `-enable-ei-portal`, so an X11 client asking for synthetic input makes Xwayland
//! request permission from the portal on that client's behalf. It requests without
//! `persist_mode`, which leaves the desktop nothing it is permitted to remember — so
//! the "Remote Desktop · Allow Remote Interaction" dialog returns on *every* paste.
//! That is the complaint this module exists to answer.
//!
//! Asking for ourselves is what makes the difference. The portal lets a client say "I
//! intend to come back": it returns a restore token, and a session started with that
//! token is granted silently. So the dialog appears once, on the first paste after
//! install, and never again — the token outlives restarts because it is written to
//! disk beside the settings.
//!
//! Dropping XTEST has a second effect worth having. XTEST input reaches Xwayland
//! clients only, so pasting into a native Wayland application never worked; the portal
//! injects at the compositor, which every window sees.

use anyhow::{anyhow, Context, Result};
use ashpd::desktop::remote_desktop::{DeviceType, KeyState, RemoteDesktop};
use ashpd::desktop::PersistMode;
use ashpd::desktop::Session;
use std::sync::Mutex;

/// X keysyms, which is what the portal's keysym call speaks. Using keysyms rather than
/// evdev keycodes keeps the paste correct on a non-QWERTY layout, where the key in the
/// physical `V` position is not the one `Ctrl+V` needs.
const XK_CONTROL_L: i32 = 0xffe3;

/// The live session, kept for the lifetime of the process.
///
/// Held open deliberately. Tearing it down after each paste would mean a fresh
/// `Start()` next time, and while the restore token keeps that from prompting, it still
/// costs a round trip through the portal on a path the user is watching.
static SESSION: Mutex<Option<PortalSession>> = Mutex::new(None);

struct PortalSession {
    proxy: RemoteDesktop<'static>,
    session: Session<'static, RemoteDesktop<'static>>,
}

// The proxy and session are plain D-Bus handles; nothing in them is tied to a thread.
unsafe impl Send for PortalSession {}

/// Presses `modifier`, clicks `ch`, and releases — the portal's equivalent of the XTEST
/// sequence in the parent module.
///
/// Returns `Err` on any portal failure, including the user declining the dialog, so the
/// caller can fall back to XTEST rather than leaving the text undelivered.
pub fn send_shortcut(app: &tauri::AppHandle, ch: char) -> Result<()> {
    let keysym = ch as i32;
    // On a borrowed thread, never the caller's.
    //
    // A paste can be dispatched from inside the app's own tokio runtime, and blocking on
    // a second runtime from a thread that is already driving one panics outright —
    // "Cannot start a runtime from within a runtime". Scoped so the portal call still
    // borrows `app` and the caller still waits for the keystroke to land.
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                runtime().block_on(async {
                    let mut guard =
                        SESSION.lock().map_err(|_| anyhow!("portal session lock"))?;
                    if guard.is_none() {
                        *guard = Some(open_session(app).await?);
                    }
                    let held = guard.as_ref().expect("just opened");

                    // A failed notify almost always means the session died under us —
                    // the desktop was restarted, or the user revoked the permission.
                    // Drop it and try once more, which re-opens with the stored token
                    // and stays silent.
                    match press(held, keysym).await {
                        Ok(()) => Ok(()),
                        Err(first) => {
                            log::debug!("portal keystroke failed, reopening: {first}");
                            *guard = Some(open_session(app).await?);
                            press(guard.as_ref().expect("just reopened"), keysym).await
                        }
                    }
                })
            })
            .join()
            .map_err(|_| anyhow!("the portal thread panicked"))?
    })
}

async fn press(held: &PortalSession, keysym: i32) -> Result<()> {
    held.proxy
        .notify_keyboard_keysym(&held.session, XK_CONTROL_L, KeyState::Pressed)
        .await
        .context("pressing Control through the portal")?;
    held.proxy
        .notify_keyboard_keysym(&held.session, keysym, KeyState::Pressed)
        .await
        .context("pressing the shortcut key through the portal")?;
    let released = held
        .proxy
        .notify_keyboard_keysym(&held.session, keysym, KeyState::Released)
        .await;
    // Control is released whatever happened to the key, exactly as in the XTEST path: a
    // modifier left down would wedge the whole desktop.
    held.proxy
        .notify_keyboard_keysym(&held.session, XK_CONTROL_L, KeyState::Released)
        .await
        .context("releasing Control through the portal")?;
    released.context("releasing the shortcut key through the portal")?;
    Ok(())
}

async fn open_session(app: &tauri::AppHandle) -> Result<PortalSession> {
    let proxy = RemoteDesktop::new()
        .await
        .context("connecting to the Remote Desktop portal")?;
    let session = proxy
        .create_session()
        .await
        .context("creating a Remote Desktop session")?;

    let token = read_token(app);
    proxy
        .select_devices(
            &session,
            DeviceType::Keyboard.into(),
            token.as_deref(),
            // The whole point: without this the desktop is not allowed to remember the
            // grant, and every paste asks again.
            PersistMode::ExplicitlyRevoked,
        )
        .await
        .context("asking the portal for keyboard access")?;

    let response = proxy
        .start(&session, None)
        .await
        .context("starting the Remote Desktop session")?
        .response()
        .context("the Remote Desktop request was declined")?;

    // Stored only when it changes: the portal reissues the token on each start, and
    // writing an identical value on every session is a pointless disk touch.
    if let Some(fresh) = response.restore_token() {
        if token.as_deref() != Some(fresh) {
            write_token(app, fresh);
        }
    }

    log::info!("Remote Desktop portal session started for pasting");
    Ok(PortalSession { proxy, session })
}

/// Where the restore token lives. Beside the settings rather than inside them: it is
/// machine-local, meaningless on another box, and a settings file is something users
/// copy between machines.
fn token_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    use tauri::Manager;
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("portal-restore-token"))
}

fn read_token(app: &tauri::AppHandle) -> Option<String> {
    let raw = std::fs::read_to_string(token_path(app)?).ok()?;
    let token = raw.trim().to_owned();
    (!token.is_empty()).then_some(token)
}

fn write_token(app: &tauri::AppHandle, token: &str) {
    let Some(path) = token_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // The token is a capability: anything holding it can resume a session that types
    // into the user's desktop, so it gets the same 0600 treatment as the key vault.
    if let Err(e) = crate::private_file::write(&path, token) {
        log::warn!("could not store the portal restore token: {e}");
    }
}

/// A runtime for the portal calls, which are async while the paste path is not.
///
/// Its own, rather than the app's: a paste may well originate inside the app's runtime,
/// and a nested `block_on` there is a panic. Multi-threaded with a single worker because
/// `block_on` on that scheduler is safe to call from several threads, which a
/// current-thread runtime does not promise.
fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("building the portal runtime")
    })
}

/// True when the portal is the right way to type here: a Wayland session, where XTEST
/// would otherwise drag in the permission dialog this module avoids.
pub fn should_use() -> bool {
    crate::hotkeys::gnome::is_wayland()
}
