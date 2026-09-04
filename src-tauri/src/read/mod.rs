//! Read-aloud session lifecycle: selection → optional AI pass → chunk → synthesize +
//! play → mini player. Mirrors `session/mod.rs`'s shape deliberately — same "every
//! entry point is safe to call redundantly" contract, same reason: hotkey handlers and
//! UI commands both call straight in with no state tracking of their own.

use crate::errors::{ErrorKind, ErrorPayload};
use crate::refine::{self, RefineRequest};
use crate::selection::{self, SelectionError};
use crate::session;
use crate::settings::{defaults, secrets, ReadMode};
use crate::state::{self, emit_error, ReadPhase, ReadStatusPayload};
use crate::{focus, player_window, tts};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};

const FIRST_CHUNK_MAX: usize = 120;
const REST_CHUNK_MAX: usize = 280;
/// Same ceiling the TTS providers themselves use per request — past this the listener
/// gave up minutes ago.
const CHUNK_TIMEOUT: Duration = Duration::from_secs(20);
/// Never synthesize more than this many chunks ahead of what has already finished
/// playing — a cancel five seconds into a ten-minute article should only have paid for
/// a couple of chunks, not the whole queue.
const MAX_QUEUED_CHUNKS: usize = 2;
const THROTTLE_POLL: Duration = Duration::from_millis(150);
/// ~4 updates/sec — smooth enough for a progress bar, light enough for IPC. Same
/// cadence `session/mod.rs` caps the recording level meter at, for the same reason.
const STATUS_TICK: Duration = Duration::from_millis(250);
const DONE_LINGER: Duration = Duration::from_millis(1500);
/// Same window `session/pipeline.rs::ERROR_VISIBLE` gives a failed dictation.
const ERROR_VISIBLE: Duration = Duration::from_secs(6);
/// A clipboard offer stays up far longer than a plain error: it is not a notice to
/// glance at but a question to answer, and six seconds is not enough to read it, decide
/// and aim at the button.
const CLIPBOARD_OFFER_VISIBLE: Duration = Duration::from_secs(30);
/// How much of the clipboard the offer shows. Enough to recognise what the text *is*
/// at a glance, short enough to stay on the one line the player has room for — and to
/// keep a password that happens to be on the clipboard from being spelled out in full
/// on a screen someone else may be looking at.
const PREVIEW_MAX_CHARS: usize = 70;

struct ReadSession {
    cancelled: Arc<AtomicBool>,
    target_app: focus::TargetApp,
    /// The text this session started from, before any AI pass. Kept so switching read
    /// mode in the player can re-run against the original selection — by then the
    /// user's highlight is long gone (the clipboard round-trip happened once, at the
    /// start) and there is nothing left to re-capture.
    source: String,
}

#[derive(Default)]
pub struct ReadState {
    active: Mutex<Option<ReadSession>>,
    /// Most recent status, so the player window can render the moment it opens instead
    /// of waiting for the next event — see [`begin_session`].
    last_status: Mutex<Option<ReadStatusPayload>>,
    /// Text the player is currently offering to read in place of a selection it could
    /// not capture, with the app to hand focus back to afterwards. Set only while that
    /// offer is on screen, and cleared the moment anything supersedes it — an offer
    /// that outlived its window would let a much later press read whatever the user had
    /// copied and forgotten about.
    pending_clipboard: Mutex<Option<(String, focus::TargetApp)>>,
    /// Held for the whole of [`start`]'s synchronous prologue. That stretch is not
    /// instant — capturing the selection polls the clipboard for up to 300ms — and
    /// until the session is registered `is_reading` still answers false, so a second
    /// press (or the tray item and the hotkey together) would otherwise sail past the
    /// toggle check and open a second session that fights the first for the audio
    /// device and the clipboard.
    starting: AtomicBool,
}

/// RAII claim on [`ReadState::starting`] — a `Drop` guard rather than a bare flag so
/// the early `return`s in `start`'s error branches cannot leave it stuck set, which
/// would wedge read-aloud until the app restarted.
struct StartGuard(AppHandle);

impl StartGuard {
    fn claim(app: &AppHandle) -> Option<Self> {
        app.state::<ReadState>()
            .starting
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .ok()
            .map(|_| Self(app.clone()))
    }
}

impl Drop for StartGuard {
    fn drop(&mut self) {
        self.0.state::<ReadState>().starting.store(false, Ordering::SeqCst);
    }
}

pub fn is_reading(app: &AppHandle) -> bool {
    app.state::<ReadState>()
        .active
        .lock()
        .map(|s| s.is_some())
        .unwrap_or(false)
}

/// Starts reading if idle, stops if a session is already running — the read hotkey's
/// only job.
pub fn toggle(app: &AppHandle, mode_id: Option<String>) {
    if is_reading(app) {
        stop(app);
    } else {
        start(app, mode_id);
    }
}

/// Ends the active read session immediately: silences audio, hides the player, hands
/// focus back to whatever the user was reading. A no-op when nothing is reading, which
/// is what lets `session::start_with_intent` call this unconditionally before every
/// dictation take — mic and speaker must never run at once.
pub fn stop(app: &AppHandle) {
    // A pending clipboard offer is a session in waiting, so it stops with everything
    // else: the ✕ button, a new read and the start of a dictation take all supersede
    // it. Without this the offer would sit there unanswered behind whatever came next.
    let dismissed_offer = take_pending_clipboard(app).is_some();
    let Some(session) = app
        .state::<ReadState>()
        .active
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
    else {
        // No session, but the player is on screen holding that offer — the ✕ has to
        // close it, and nothing else here would.
        if dismissed_offer {
            forget_clipboard_preview(app);
            player_window::hide(app);
        }
        return;
    };
    session.cancelled.store(true, Ordering::Relaxed);
    app.state::<tts::player::Player>()
        .send(tts::player::Command::Stop);
    player_window::hide(app);
    focus::restore_and_settle(session.target_app);
}

pub fn pause(app: &AppHandle) {
    app.state::<tts::player::Player>()
        .send(tts::player::Command::Pause);
    restore_focus(app);
}

pub fn resume(app: &AppHandle) {
    app.state::<tts::player::Player>()
        .send(tts::player::Command::Resume);
    restore_focus(app);
}

fn restore_focus(app: &AppHandle) {
    if let Some(target) = app
        .state::<ReadState>()
        .active
        .lock()
        .ok()
        .and_then(|s| s.as_ref().map(|s| s.target_app))
    {
        focus::restore_and_settle(target);
    }
}

/// The other half of "mic and speaker never run at once": `session::start_with_intent`
/// stops an active read session before opening the mic, so this stops an active
/// dictation take before starting to speak — otherwise the read-aloud audio would play
/// straight into a live microphone and corrupt whatever is being transcribed. Discards
/// the take rather than finishing it; pressing the read key is as clear a change of
/// intent as pressing cancel would be.
fn stop_any_dictation(app: &AppHandle) {
    if app.state::<session::Recorder>().is_recording() {
        session::cancel(app);
    }
}

/// Persists which read mode is active, the read-mode equivalent of
/// `session::set_active_mode`.
fn set_active_read_mode(app: &AppHandle, mode_id: &str) {
    let state = app.state::<crate::state::AppState>();
    let updated = {
        let Ok(mut settings) = state.settings.lock() else { return };
        if settings.read_mode(mode_id).is_none() {
            return;
        }
        settings.active_read_mode_id = mode_id.to_string();
        settings.clone()
    };
    if let Err(e) = crate::settings::save(app, &updated) {
        log::warn!("could not persist the active read mode: {e:#}");
    }
    let _ = tauri::Emitter::emit(app, state::events::SETTINGS_CHANGED, ());
}

/// Flow A/B: reads whatever is currently highlighted, in `mode_id`'s mode (or the
/// already-active one). Pressing the bound key a second time stops instead of starting
/// a second session.
pub fn start(app: &AppHandle, mode_id: Option<String>) {
    if is_reading(app) {
        stop(app);
        return;
    }
    // Claimed for the whole prologue below and released by `StartGuard` on every exit
    // path, including the early returns.
    let Some(_guard) = StartGuard::claim(app) else {
        log::debug!("a read is already starting; ignoring the repeat");
        return;
    };
    stop_any_dictation(app);

    if let Some(id) = &mode_id {
        set_active_read_mode(app, id);
    }

    let settings = state::settings_snapshot(app);
    if !settings.tts.enabled {
        return;
    }
    let mode = settings.active_read_mode().clone();

    let target_app = focus::capture();

    // Nothing of ours may appear on screen until the selection is in hand. Showing a
    // window here — even one declared `focus: false` and never given `set_focus` — can
    // put us in front at the exact moment the copy keystroke is synthesized, and then
    // the copy lands on the player instead of on the app being read. That is what made
    // this fail in Terminal. `fail` shows the player by itself, so the error branches
    // below are still visible even though nothing is on screen yet.
    let selection = match selection::capture(app) {
        Ok(text) => text,
        Err(SelectionError::NothingSelected { clipboard }) => {
            // Also reached from the tray: opening that menu makes us frontmost, so the
            // synthesized copy lands on our own app and comes back empty.
            let kind = if target_app.can_receive_paste() {
                ErrorKind::NothingSelected
            } else {
                ErrorKind::NoPasteTarget
            };
            // Whatever is already on the clipboard is the way out of the apps that will
            // not service a synthesized copy at all (terminals, most notably): offer to
            // read that rather than dead-ending on "nothing is highlighted", which is
            // untrue there and leaves the user with nothing to try.
            fail_with_offer(app, target_app, ErrorPayload::new(kind), clipboard);
            return;
        }
        Err(SelectionError::NeedsAccessibility) => {
            fail(app, target_app, ErrorPayload::new(ErrorKind::NeedsAccessibility));
            return;
        }
        Err(SelectionError::Clipboard(e)) => {
            fail(
                app,
                target_app,
                ErrorPayload::with_detail(ErrorKind::DeliveryFailed, format!("{e:#}")),
            );
            return;
        }
    };

    launch(app, selection, target_app, settings, mode);
}

/// Reads what the failed capture found on the clipboard — the player's "read the
/// clipboard instead?" button. A no-op unless that offer is still standing, so a stale
/// click (or a command fired at nothing) cannot resurrect text the user copied long ago.
pub fn start_from_clipboard(app: &AppHandle) {
    let Some((text, target_app)) = take_pending_clipboard(app) else {
        log::debug!("no clipboard offer is standing; ignoring the request to read it");
        return;
    };
    let settings = state::settings_snapshot(app);
    if !settings.tts.enabled {
        return;
    }
    let mode = settings.active_read_mode().clone();
    launch(app, text, target_app, settings, mode);
}

/// Everything from "the text is in hand" onwards: the guards, the player, then the
/// optional AI pass and the speaking itself. Shared by the selection path and the
/// clipboard fallback so both are held to the same limits and look identical on screen.
fn launch(
    app: &AppHandle,
    text: String,
    target_app: focus::TargetApp,
    settings: crate::settings::AppSettings,
    mode: ReadMode,
) {
    if let Some(payload) = check_length(&text, settings.tts.max_chars) {
        fail(app, target_app, payload);
        return;
    }
    if secrets::get_key(tts::secret_account(settings.tts.provider)).is_none() {
        fail(
            app,
            target_app,
            ErrorPayload::with_detail(ErrorKind::NoTtsKey, format!("{:?}", settings.tts.provider)),
        );
        return;
    }

    // Now that the clipboard round-trip is done, the player can go up — still well
    // before any audio, since the AI pass and the first synthesis are what actually
    // take seconds.
    begin_session(app, target_app, &mode.name);
    remember_source(app, &text);

    let app = app.clone();
    let mode_label = mode.name.clone();
    tauri::async_runtime::spawn(async move {
        let text = if mode.ai {
            match run_ai_pass(&settings, &mode, &text).await {
                Ok(t) => t,
                Err(payload) => {
                    fail(&app, target_app, payload);
                    return;
                }
            }
        } else {
            text
        };
        speak(&app, text, mode_label, target_app).await;
    });
}

fn store_pending_clipboard(app: &AppHandle, text: String, target_app: focus::TargetApp) {
    if let Ok(mut slot) = app.state::<ReadState>().pending_clipboard.lock() {
        *slot = Some((text, target_app));
    }
}

/// Takes the standing offer, if any — reading it always consumes it, so a second press
/// of the button (or a stale one after the player has moved on) does nothing.
fn take_pending_clipboard(app: &AppHandle) -> Option<(String, focus::TargetApp)> {
    app.state::<ReadState>().pending_clipboard.lock().ok()?.take()
}

/// Flow C entry point: capture the selection, then take a spoken instruction about it
/// through the ordinary dictation take (`session::start_with_intent`). The result is
/// handled by `session/pipeline.rs::finish_read_instruction`, which calls
/// [`speak_from_flow_c`] back into this module once the LLM has answered.
pub fn start_voice_command(app: &AppHandle) {
    let Some(_guard) = StartGuard::claim(app) else {
        log::debug!("a read is already starting; ignoring the repeat");
        return;
    };
    let settings = state::settings_snapshot(app);
    if !settings.tts.enabled {
        return;
    }
    // Replaces whatever was already being read, same as pressing the plain read key
    // again — a new request from the user always wins.
    if is_reading(app) {
        stop(app);
    }
    stop_any_dictation(app);

    let selection = match selection::capture(app) {
        Ok(text) => text,
        // No clipboard fallback here, unlike the plain read: this flow takes a spoken
        // instruction *about* the highlighted passage, and answering it against
        // whatever happens to be on the clipboard would silently be about something
        // else entirely.
        Err(SelectionError::NothingSelected { .. }) => {
            emit_error(app, ErrorPayload::new(ErrorKind::NothingSelected));
            return;
        }
        Err(SelectionError::NeedsAccessibility) => {
            emit_error(app, ErrorPayload::new(ErrorKind::NeedsAccessibility));
            return;
        }
        Err(SelectionError::Clipboard(e)) => {
            emit_error(app, ErrorPayload::with_detail(ErrorKind::DeliveryFailed, format!("{e:#}")));
            return;
        }
    };
    if let Some(payload) = check_length(&selection, settings.tts.max_chars) {
        emit_error(app, payload);
        return;
    }

    crate::session::start_with_intent(app, None, crate::session::Intent::ReadInstruction { selection });
}

/// Called by `pipeline::finish_read_instruction` once flow C's LLM answer (or the raw
/// selection, if the spoken instruction was empty) is ready to be heard.
pub fn speak_from_flow_c(app: &AppHandle, text: String, target_app: focus::TargetApp) {
    let settings = state::settings_snapshot(app);
    if !settings.tts.enabled {
        focus::restore_and_settle(target_app);
        return;
    }
    if secrets::get_key(tts::secret_account(settings.tts.provider)).is_none() {
        fail(
            app,
            target_app,
            ErrorPayload::with_detail(ErrorKind::NoTtsKey, format!("{:?}", settings.tts.provider)),
        );
        return;
    }

    const FLOW_C_LABEL: &str = "Ra lệnh bằng giọng nói";
    begin_session(app, target_app, FLOW_C_LABEL);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        speak(&app, text, FLOW_C_LABEL.to_string(), target_app).await;
    });
}

fn check_length(selection: &str, max_chars: usize) -> Option<ErrorPayload> {
    let len = selection.chars().count();
    if len > max_chars {
        Some(ErrorPayload::with_detail(
            ErrorKind::SelectionTooLong,
            format!("{len} / {max_chars} ký tự"),
        ))
    } else {
        None
    }
}

/// Registers the session, shows the player, and announces `Preparing` — done as one
/// step so every path that starts speaking (flow A/B and flow C) looks the same to the
/// player from the very first frame, before any AI or synthesis call has even begun.
fn begin_session(app: &AppHandle, target_app: focus::TargetApp, mode_name: &str) -> Arc<AtomicBool> {
    let cancelled = Arc::new(AtomicBool::new(false));
    if let Ok(mut slot) = app.state::<ReadState>().active.lock() {
        *slot = Some(ReadSession {
            cancelled: cancelled.clone(),
            target_app,
            source: String::new(),
        });
    }
    let settings = state::settings_snapshot(app);
    player_window::show(app, settings.tts.player_drag_position);
    let payload = ReadStatusPayload {
        phase: ReadPhase::Preparing,
        mode_name: mode_name.to_string(),
        provider: format!("{:?}", settings.tts.provider),
        speed: settings.tts.speed,
        speed_min: speed_range(settings.tts.provider).0,
        speed_max: speed_range(settings.tts.provider).1,
        played_ms: 0,
        total_ms: 0,
        error: None,
        clipboard_preview: None,
    };
    // Also parked as the "latest" status so the player can ask for it on mount: the
    // window's webview subscribes to the event stream only once it has booted, which
    // on the very first read happens *after* this fires. Without the snapshot the
    // player would sit blank until the first Speaking tick — i.e. until audio was
    // already playing, which is exactly backwards from what it is for.
    store_last_status(app, payload.clone());
    state::emit_read_status(app, payload);
    cancelled
}

fn remember_source(app: &AppHandle, source: &str) {
    if let Ok(mut slot) = app.state::<ReadState>().active.lock() {
        if let Some(session) = slot.as_mut() {
            session.source = source.to_string();
        }
    }
}

/// Re-reads the current session's original selection under a different mode, driven by
/// the player's mode dropdown. Falls back to simply switching the active mode when
/// nothing is being read, so the choice still sticks for next time.
pub fn switch_mode(app: &AppHandle, mode_id: String) {
    let source = app
        .state::<ReadState>()
        .active
        .lock()
        .ok()
        .and_then(|s| s.as_ref().map(|s| (s.source.clone(), s.target_app)));

    set_active_read_mode(app, &mode_id);

    let Some((source, target_app)) = source.filter(|(text, _)| !text.trim().is_empty()) else {
        return;
    };

    stop(app);

    let settings = state::settings_snapshot(app);
    let Some(mode) = settings.read_mode(&mode_id).cloned() else { return };
    begin_session(app, target_app, &mode.name);
    remember_source(app, &source);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let text = if mode.ai {
            match run_ai_pass(&settings, &mode, &source).await {
                Ok(t) => t,
                Err(payload) => {
                    fail(&app, target_app, payload);
                    return;
                }
            }
        } else {
            source
        };
        speak(&app, text, mode.name.clone(), target_app).await;
    });
}

/// Keeps the most recent status so a freshly-opened player can render immediately
/// rather than waiting for the next tick. See [`begin_session`].
fn store_last_status(app: &AppHandle, payload: ReadStatusPayload) {
    if let Ok(mut slot) = app.state::<ReadState>().last_status.lock() {
        *slot = Some(payload);
    }
}

/// The current read status, for the player window to pull on mount.
pub fn last_status(app: &AppHandle) -> Option<ReadStatusPayload> {
    app.state::<ReadState>().last_status.lock().ok()?.clone()
}

async fn run_ai_pass(
    settings: &crate::settings::AppSettings,
    mode: &ReadMode,
    selection: &str,
) -> Result<String, ErrorPayload> {
    let llm = settings.llm_for_read_mode(mode);
    let api_key = defaults::preset(&llm.preset)
        .filter(|p| p.needs_key)
        .and_then(|p| secrets::get_key(p.secret_key));

    if defaults::preset(&llm.preset).map(|p| p.needs_key).unwrap_or(true) && api_key.is_none() {
        return Err(ErrorPayload::with_detail(ErrorKind::NoLlmKey, llm.preset.clone()));
    }

    let request = RefineRequest {
        system_prompt: mode.prompt.clone(),
        transcript: selection.to_string(),
        llm,
        api_key,
    };

    match refine::refine(request).await {
        Ok(text) if !text.trim().is_empty() => Ok(text),
        Ok(_) => Err(ErrorPayload::new(ErrorKind::CleanupFailed)),
        Err(e) => Err(ErrorPayload::with_detail(ErrorKind::CleanupFailed, format!("{e:#}"))),
    }
}

/// Chunks, synthesizes and plays `text`, keeping the player's status roughly current
/// throughout. Cancellation (`stop`) is checked between every chunk and while waiting
/// on the throttle, so a mid-article cancel stops synthesizing almost immediately
/// rather than draining the whole queue first.
async fn speak(app: &AppHandle, text: String, mode_label: String, target_app: focus::TargetApp) {
    let settings = state::settings_snapshot(app);
    let Some(api_key) = secrets::get_key(tts::secret_account(settings.tts.provider)) else {
        fail(
            app,
            target_app,
            ErrorPayload::with_detail(ErrorKind::NoTtsKey, format!("{:?}", settings.tts.provider)),
        );
        return;
    };
    let engine = tts::build_engine(&settings.tts, api_key);

    let chunks = tts::chunker::split(&text, FIRST_CHUNK_MAX, REST_CHUNK_MAX);
    if chunks.is_empty() {
        finish_naturally(app, target_app).await;
        return;
    }

    let Some(cancelled) = app
        .state::<ReadState>()
        .active
        .lock()
        .ok()
        .and_then(|s| s.as_ref().map(|s| s.cancelled.clone()))
    else {
        return; // stopped before synthesis even began
    };

    let player = app.state::<tts::player::Player>();
    if let Err(e) = player.start() {
        fail(app, target_app, ErrorPayload::with_detail(ErrorKind::TtsFailed, format!("{e:#}")));
        return;
    }

    let ticker = spawn_status_ticker(app, cancelled.clone(), mode_label, &settings);

    let mut cumulative_ms: u64 = 0;
    let mut boundaries: Vec<u64> = Vec::new();
    let mut failure: Option<ErrorPayload> = None;

    for chunk_text in chunks {
        if cancelled.load(Ordering::Relaxed) {
            break;
        }

        while boundaries.len() >= MAX_QUEUED_CHUNKS {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            let played = player.progress().played_ms;
            boundaries.retain(|&b| b > played);
            if boundaries.len() < MAX_QUEUED_CHUNKS {
                break;
            }
            tokio::time::sleep(THROTTLE_POLL).await;
        }
        if cancelled.load(Ordering::Relaxed) {
            break;
        }

        let req = tts::SpeechRequest {
            text: chunk_text,
            voice: settings.tts.voice.clone(),
            speed: settings.tts.speed,
        };
        let audio = match tokio::time::timeout(CHUNK_TIMEOUT, tts::synthesize(&engine, &req)).await {
            Ok(Ok(audio)) => audio,
            Ok(Err(e)) => {
                failure = Some(ErrorPayload::with_detail(ErrorKind::TtsFailed, format!("{e:#}")));
                break;
            }
            Err(_) => {
                failure = Some(ErrorPayload::with_detail(
                    ErrorKind::TtsFailed,
                    format!("no response after {}s", CHUNK_TIMEOUT.as_secs()),
                ));
                break;
            }
        };

        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        cumulative_ms += tts::player::duration_ms(&audio);
        boundaries.push(cumulative_ms);
        player.send(tts::player::Command::Enqueue(audio));
    }

    ticker.abort();

    if let Some(payload) = failure {
        player.send(tts::player::Command::Stop);
        fail(app, target_app, payload);
        return;
    }
    if cancelled.load(Ordering::Relaxed) {
        return; // `stop` already cleared the session and hid the player
    }

    while !cancelled.load(Ordering::Relaxed) {
        let progress = player.progress();
        if progress.total_ms > 0 && progress.played_ms >= progress.total_ms {
            break;
        }
        tokio::time::sleep(THROTTLE_POLL).await;
    }
    if !cancelled.load(Ordering::Relaxed) {
        finish_naturally(app, target_app).await;
    }
}

fn spawn_status_ticker(
    app: &AppHandle,
    cancelled: Arc<AtomicBool>,
    mode_label: String,
    settings: &crate::settings::AppSettings,
) -> tauri::async_runtime::JoinHandle<()> {
    let app = app.clone();
    let provider = format!("{:?}", settings.tts.provider);
    let range = speed_range(settings.tts.provider);
    tauri::async_runtime::spawn(async move {
        while !cancelled.load(Ordering::Relaxed) {
            let progress = app.state::<tts::player::Player>().progress();
            // Still `Preparing` until a chunk has actually been queued. The ticker
            // starts the moment synthesis begins, so reporting `Speaking` straight
            // away overwrote the preparing state within one tick — the loading bar
            // vanished before it could be seen and was replaced by a frozen
            // "0:00 / 0:00", which reads as the app having done nothing at all.
            let phase = if progress.total_ms == 0 {
                ReadPhase::Preparing
            } else if progress.paused {
                ReadPhase::Paused
            } else {
                ReadPhase::Speaking
            };
            publish(
                &app,
                ReadStatusPayload {
                    phase,
                    mode_name: mode_label.clone(),
                    provider: provider.clone(),
                    // Re-read every tick rather than captured once: the player's own
                    // speed control edits this mid-session, and a stale copy would
                    // make the slider snap back on the next tick.
                    speed: state::settings_snapshot(&app).tts.speed,
                    speed_min: range.0,
                    speed_max: range.1,
                    played_ms: progress.played_ms,
                    total_ms: progress.total_ms,
                    error: None,
                    clipboard_preview: None,
                },
            );
            tokio::time::sleep(STATUS_TICK).await;
        }
    })
}

/// The active provider's accepted speed range, mirrored into every status payload so
/// the player's slider stays inside it. Static per provider rather than fetched — the
/// player must not make a network call to draw a slider, and these match the ranges
/// each provider's own catalogue reports.
fn speed_range(provider: crate::settings::TtsProviderKind) -> (f32, f32) {
    use crate::settings::TtsProviderKind as P;
    match provider {
        P::Soniox => (0.7, 1.3),
        P::OpenAi => (0.25, 4.0),
        P::ElevenLabs => (0.7, 1.2),
        P::Vbee => (0.25, 1.9),
        // No speed parameter at all; a degenerate range keeps the slider pinned.
        P::Gemini => (1.0, 1.0),
    }
}

/// Emits a status and keeps it as the latest, so a player window that opens (or
/// reloads) afterwards can render it immediately instead of waiting for the next tick.
fn publish(app: &AppHandle, payload: ReadStatusPayload) {
    store_last_status(app, payload.clone());
    state::emit_read_status(app, payload);
}

async fn finish_naturally(app: &AppHandle, target_app: focus::TargetApp) {
    let settings = state::settings_snapshot(app);
    publish(
        app,
        ReadStatusPayload {
            phase: ReadPhase::Done,
            mode_name: String::new(),
            provider: format!("{:?}", settings.tts.provider),
            speed: settings.tts.speed,
            speed_min: speed_range(settings.tts.provider).0,
            speed_max: speed_range(settings.tts.provider).1,
            played_ms: 0,
            total_ms: 0,
            error: None,
            clipboard_preview: None,
        },
    );
    tokio::time::sleep(DONE_LINGER).await;
    if let Ok(mut slot) = app.state::<ReadState>().active.lock() {
        slot.take();
    }
    player_window::hide(app);
    focus::restore_and_settle(target_app);
}

/// Shared failure path: clears the session, shows the error in the player for
/// [`ERROR_VISIBLE`], then hides it. Used by every failure branch in this module so a
/// dead end always looks the same from the player's side.
pub fn fail(app: &AppHandle, target_app: focus::TargetApp, payload: ErrorPayload) {
    fail_with_offer(app, target_app, payload, None);
}

/// [`fail`], plus an offer to read `clipboard` instead when there is something there to
/// offer. The offer turns a dead end into one button press for the apps a synthesized
/// copy never reaches; with `None` this behaves exactly like a plain failure.
fn fail_with_offer(
    app: &AppHandle,
    target_app: focus::TargetApp,
    payload: ErrorPayload,
    clipboard: Option<String>,
) {
    if let Ok(mut slot) = app.state::<ReadState>().active.lock() {
        slot.take();
    }
    emit_error(app, payload.clone());
    let preview = clipboard.as_deref().map(preview_of);
    if let Some(text) = clipboard {
        // Length only, never content — same rule as `selection::capture`'s logging.
        // The preview goes to the screen the user is already looking at; a log file
        // gets copied into bug reports, so it stays out of there.
        log::info!("offering to read {} chars from the clipboard instead", text.len());
        store_pending_clipboard(app, text, target_app);
    }
    // Make sure there is somewhere to read this. A read started from the tray or a
    // hotkey may have no window on screen at all, and an error sent only to the event
    // bus in that state is an error nobody ever sees — which is precisely how "the
    // tray item does nothing" looked.
    let settings = state::settings_snapshot(app);
    player_window::show(app, settings.tts.player_drag_position);
    publish(
        app,
        ReadStatusPayload {
            phase: ReadPhase::Failed,
            mode_name: String::new(),
            provider: String::new(),
            speed: settings.tts.speed,
            speed_min: speed_range(settings.tts.provider).0,
            speed_max: speed_range(settings.tts.provider).1,
            played_ms: 0,
            total_ms: 0,
            error: Some(payload),
            clipboard_preview: preview.clone(),
        },
    );
    focus::restore_and_settle(target_app);

    let visible = if preview.is_some() { CLIPBOARD_OFFER_VISIBLE } else { ERROR_VISIBLE };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(visible).await;
        if !is_reading(&app) {
            // The offer expires with the window it was made in, so a press minutes
            // later cannot read text the user has long since stopped thinking about.
            if take_pending_clipboard(&app).is_some() {
                forget_clipboard_preview(&app);
            }
            player_window::hide(&app);
        }
    });
}

/// The first line or so of `text`, for the offer to show. Whitespace is flattened
/// because a clipboard full of newlines would otherwise render as one visible word, and
/// the cut counts *characters*, not bytes — a byte slice would land mid-codepoint on
/// Vietnamese and panic.
fn preview_of(text: &str) -> String {
    let flattened = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview: String = flattened.chars().take(PREVIEW_MAX_CHARS).collect();
    if flattened.chars().count() > PREVIEW_MAX_CHARS {
        preview.push('…');
    }
    preview
}

/// Drops the preview from the stored status once the offer behind it is gone, so a
/// snapshot of the user's clipboard does not sit in memory long after the question it
/// was answering stopped being asked.
fn forget_clipboard_preview(app: &AppHandle) {
    if let Ok(mut slot) = app.state::<ReadState>().last_status.lock() {
        if let Some(status) = slot.as_mut() {
            status.clipboard_preview = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_selection_within_the_limit_passes() {
        assert!(check_length("hello", 10).is_none());
    }

    #[test]
    fn a_short_clipboard_previews_whole_and_unmarked() {
        assert_eq!(preview_of("xin chào"), "xin chào");
    }

    #[test]
    fn a_long_clipboard_is_cut_to_the_limit_and_marked() {
        let preview = preview_of(&"a".repeat(200));
        assert_eq!(preview.chars().count(), PREVIEW_MAX_CHARS + 1);
        assert!(preview.ends_with('…'));
    }

    /// A byte-based cut would split a Vietnamese character here and panic.
    #[test]
    fn cutting_multibyte_text_counts_characters_not_bytes() {
        let preview = preview_of(&"ườ".repeat(100));
        assert_eq!(preview.chars().count(), PREVIEW_MAX_CHARS + 1);
    }

    #[test]
    fn newlines_and_runs_of_spaces_collapse_so_the_preview_stays_one_line() {
        assert_eq!(preview_of("dòng một\n\n  dòng hai\t"), "dòng một dòng hai");
    }

    #[test]
    fn a_selection_over_the_limit_is_rejected_with_the_count() {
        let payload = check_length(&"a".repeat(20), 10).unwrap();
        assert!(matches!(payload.kind, ErrorKind::SelectionTooLong));
        assert_eq!(payload.detail.as_deref(), Some("20 / 10 ký tự"));
    }
}
