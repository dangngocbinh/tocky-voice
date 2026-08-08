//! Dictation session lifecycle: start → capture + stream → stop → transcript → paste.
//!
//! Every entry point here is callable from a hotkey handler or a UI command and is
//! safe to invoke redundantly (starting while recording, stopping while idle) — the
//! guards live here so callers don't have to track state themselves.

mod pipeline;

use crate::audio::{capture, feedback, mic_test, resample};
use crate::focus;
use crate::overlay;
use crate::settings::secrets;
use crate::errors::{ErrorKind, ErrorPayload};
use crate::state::{self, emit_error, events, Phase};
use crate::stt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

/// A recording in flight.
struct ActiveTake {
    capture: capture::CaptureHandle,
    /// Dropping this closes the audio channel, which is how the STT stream is told
    /// that no more audio is coming.
    audio_tx: Option<mpsc::UnboundedSender<Vec<u8>>>,
    /// Set when the user aborts, so the pipeline knows to throw the result away.
    cancelled: Arc<AtomicBool>,
    /// Set the first time a chunk arrives above [`AUDIBLE_PEAK`]. An empty transcript
    /// means something very different depending on this flag: either nothing was said,
    /// or the selected input never produced a sound in the first place.
    heard_audio: Arc<AtomicBool>,
    pcm: Arc<Mutex<Vec<i16>>>,
    mode_id: String,
    /// The app that was frontmost when recording began — the one the text belongs in,
    /// even if the user clicks the overlay before stopping.
    target_app: focus::TargetApp,
    stt_task: tauri::async_runtime::JoinHandle<anyhow::Result<String>>,
}

/// Peak amplitude (0..1) a chunk has to reach to count as "the microphone is live".
/// A muted or disconnected Windows input hands back exact zeros or near-zeros; even a
/// quiet room on a low-gain mic sits above this.
const AUDIBLE_PEAK: f32 = 0.01;

#[derive(Default)]
pub struct Recorder {
    active: Mutex<Option<ActiveTake>>,
}

impl Recorder {
    pub fn is_recording(&self) -> bool {
        self.active
            .lock()
            .map(|slot| slot.is_some())
            .unwrap_or(false)
    }
}

pub fn toggle(app: &AppHandle) {
    if app.state::<Recorder>().is_recording() {
        stop(app);
    } else {
        start(app, None);
    }
}

/// Begins recording. `mode_id` switches mode first; `None` keeps the active one.
/// A no-op if a take is already running, so a repeated key press is harmless.
pub fn start(app: &AppHandle, mode_id: Option<String>) {
    let recorder = app.state::<Recorder>();
    if recorder.is_recording() {
        return;
    }

    // The setup wizard's level meter holds the device open. Some Windows drivers give
    // exclusive access, so a preview left running would make the real take fail to open
    // the very microphone it just proved was working.
    mic_test::stop(app);

    if let Some(id) = mode_id {
        set_active_mode(app, &id);
    }

    let settings = state::settings_snapshot(app);
    let mode_id = settings.active_mode().id.clone();

    // Capture the target before anything of ours can take focus.
    let target_app = focus::capture();

    let Some(api_key) = secrets::get_key(secrets::stt_account(&settings.stt.provider)) else {
        emit_error(
            app,
            ErrorPayload::with_detail(ErrorKind::NoSttKey, format!("{:?}", settings.stt.provider)),
        );
        feedback::play(feedback::Cue::Error, settings.audio.feedback_volume);
        return;
    };

    let (chunk_tx, mut chunk_rx) = mpsc::unbounded_channel::<capture::CaptureChunk>();
    let capture = match capture::start(settings.audio.input_device.clone(), chunk_tx) {
        Ok(handle) => handle,
        Err(e) => {
            emit_error(app, ErrorPayload::with_detail(ErrorKind::MicUnavailable, format!("{e:#}")));
            feedback::play(feedback::Cue::Error, settings.audio.feedback_volume);
            return;
        }
    };

    let (audio_tx, audio_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<stt::SttEvent>();
    let pcm = Arc::new(Mutex::new(Vec::<i16>::new()));
    let heard_audio = Arc::new(AtomicBool::new(false));

    let pump_pcm = pcm.clone();
    let pump_heard = heard_audio.clone();
    let pump_audio_tx = audio_tx.clone();

    // Live transcript preview.
    {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    stt::SttEvent::Partial(text) => {
                        let _ = app.emit(events::PARTIAL, text);
                    }
                    stt::SttEvent::Final(text) => {
                        let _ = app.emit(events::TRANSCRIPT, text);
                    }
                }
            }
        });
    }

    let protocol = stt::build_protocol(&settings.stt, api_key);
    let stt_task = tauri::async_runtime::spawn(stt::run_stream(protocol, audio_rx, event_tx));

    if let Ok(mut slot) = recorder.active.lock() {
        *slot = Some(ActiveTake {
            capture,
            audio_tx: Some(audio_tx),
            cancelled: Arc::new(AtomicBool::new(false)),
            heard_audio,
            pcm,
            mode_id: mode_id.clone(),
            target_app,
            stt_task,
        });
    }

    // Pump: capture thread → PCM archive + level meter → provider socket.
    //
    // Spawned only once the take is recorded above, so that if the provider stream dies
    // immediately there is a take for `stop` to end.
    {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut since_last_level = 0u8;
            while let Some(chunk) = chunk_rx.recv().await {
                if let Ok(mut buffer) = pump_pcm.lock() {
                    buffer.extend_from_slice(&chunk.pcm16);
                }
                if chunk.peak >= AUDIBLE_PEAK {
                    pump_heard.store(true, Ordering::Relaxed);
                }
                if pump_audio_tx
                    .send(resample::i16_to_le_bytes(&chunk.pcm16))
                    .is_err()
                {
                    // The only way this fails is the provider stream having returned.
                    // Left alone the overlay keeps its timer running over a frozen
                    // meter and the user finds out minutes later, so end the take now
                    // and let `stop` report why the stream actually died.
                    log::warn!("speech stream ended mid-take; stopping");
                    stop(&app);
                    return;
                }
                // Roughly 10 updates/sec is plenty for a meter and keeps IPC quiet.
                since_last_level += 1;
                if since_last_level >= 3 {
                    since_last_level = 0;
                    let _ = app.emit(events::LEVEL, chunk.peak);
                }
            }
        });
    }

    let _ = app.emit(events::PARTIAL, String::new());
    overlay::show(app);
    feedback::play(feedback::Cue::Start, settings.audio.feedback_volume);
    state::emit_status(app, Phase::Recording, &mode_id);
}

/// Ends the take and hands the result to the transcript → refine → paste pipeline.
pub fn stop(app: &AppHandle) {
    let Some(mut take) = app
        .state::<Recorder>()
        .active
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
    else {
        return;
    };

    take.capture.stop();
    // Closing the channel is the signal that finalizes the provider stream.
    take.audio_tx.take();

    let settings = state::settings_snapshot(app);
    feedback::play(feedback::Cue::Stop, settings.audio.feedback_volume);
    state::emit_status(app, Phase::Transcribing, &take.mode_id);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let transcript = match take.stt_task.await {
            Ok(Ok(text)) => text,
            Ok(Err(e)) => {
                pipeline::fail(
                    &app,
                    &take.mode_id,
                    ErrorPayload::with_detail(ErrorKind::TranscriptionFailed, format!("{e:#}")),
                );
                return;
            }
            Err(e) => {
                pipeline::fail(
                    &app,
                    &take.mode_id,
                    ErrorPayload::with_detail(ErrorKind::TranscriptionFailed, format!("{e}")),
                );
                return;
            }
        };

        if take.cancelled.load(Ordering::Relaxed) {
            state::emit_status(&app, Phase::Idle, &take.mode_id);
            return;
        }

        let pcm = take
            .pcm
            .lock()
            .map(|buffer| buffer.clone())
            .unwrap_or_default();
        pipeline::finish(
            &app,
            &take.mode_id,
            transcript,
            pcm,
            take.target_app,
            take.heard_audio.load(Ordering::Relaxed),
        )
        .await;
    });
}

/// Aborts the current take and discards its audio.
pub fn cancel(app: &AppHandle) {
    let Some(mut take) = app
        .state::<Recorder>()
        .active
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
    else {
        return;
    };

    take.cancelled.store(true, Ordering::Relaxed);
    take.capture.stop();
    take.audio_tx.take();
    take.stt_task.abort();

    let settings = state::settings_snapshot(app);
    overlay::hide(app);
    feedback::play(feedback::Cue::Cancel, settings.audio.feedback_volume);
    let _ = app.emit(events::PARTIAL, String::new());
    state::emit_status(app, Phase::Idle, &take.mode_id);
}

/// Cycles to the next mode without recording — handy for a "switch mode" hotkey.
pub fn next_mode(app: &AppHandle) {
    let settings = state::settings_snapshot(app);
    if settings.modes.len() < 2 {
        return;
    }
    let current = settings
        .modes
        .iter()
        .position(|m| m.id == settings.active_mode_id)
        .unwrap_or(0);
    let next = &settings.modes[(current + 1) % settings.modes.len()];
    set_active_mode(app, &next.id.clone());
}

pub fn set_active_mode(app: &AppHandle, mode_id: &str) {
    let state = app.state::<crate::state::AppState>();
    let updated = {
        let Ok(mut settings) = state.settings.lock() else {
            return;
        };
        if settings.mode(mode_id).is_none() {
            return;
        }
        settings.active_mode_id = mode_id.to_string();
        settings.clone()
    };
    if let Err(e) = crate::settings::save(app, &updated) {
        log::warn!("could not persist active mode: {e:#}");
    }
    let _ = app.emit(events::SETTINGS_CHANGED, ());
    state::emit_status(app, Phase::Idle, mode_id);
}
