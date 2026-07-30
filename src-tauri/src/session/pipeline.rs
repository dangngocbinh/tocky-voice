//! What happens after the microphone stops: optional AI cleanup, delivery into the
//! focused app, and the history record.
//!
//! The guiding rule is that a failure here must never cost the user their words —
//! if cleanup fails we paste the raw transcript, and the history entry is written
//! even when delivery fails, so nothing is ever lost silently.

use crate::audio::feedback;
use crate::history::{self, HistoryEntry};
use crate::overlay;
use crate::refine::{self, RefineRequest};
use crate::settings::{secrets, defaults, OutputAction};
use crate::errors::{ErrorKind, ErrorPayload};
use crate::state::{self, emit_error, events, Phase};
use crate::{audio, inject};
use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager};

/// How long a failure stays on screen.
///
/// The overlay is hidden *before* delivery so the paste lands in the app the user was
/// typing in rather than in us — which means anything that goes wrong afterwards would
/// otherwise be announced into an invisible window. The symptom is nasty: an error cue
/// with no message, and then a flash of the stale text at the start of the next take,
/// gone before it can be read.
const ERROR_VISIBLE: std::time::Duration = std::time::Duration::from_secs(6);

/// Surfaces a failure where it can actually be read.
fn show_error(app: &AppHandle, payload: ErrorPayload) {
    emit_error(app, payload);
    overlay::show(app);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(ERROR_VISIBLE).await;
        // A new take may have started meanwhile — that overlay is not ours to hide.
        if !app.state::<super::Recorder>().is_recording() {
            overlay::hide(&app);
        }
    });
}

pub async fn finish(
    app: &AppHandle,
    mode_id: &str,
    transcript: String,
    pcm: Vec<i16>,
    target_app: crate::focus::TargetApp,
    heard_audio: bool,
) {
    let settings = state::settings_snapshot(app);
    let mode = match settings.mode(mode_id) {
        Some(m) => m.clone(),
        None => settings.active_mode().clone(),
    };

    if transcript.trim().is_empty() {
        if heard_audio {
            // Someone pressed the key and said nothing, or the take was too short to
            // land a word. A quiet no-op, not an error toast.
            overlay::hide(app);
            state::emit_status(app, Phase::Idle, mode_id);
        } else {
            // The microphone opened but never produced a sound: it is muted, the OS
            // privacy switch is off, or the chosen device is not the one being spoken
            // into. Silently closing the panel here is what made this look like the
            // app "just stops after two seconds".
            fail(app, mode_id, ErrorPayload::new(ErrorKind::NoAudioCaptured));
        }
        return;
    }

    let final_text = if mode.ai_cleanup {
        state::emit_status(app, Phase::Refining, mode_id);
        refine_or_fall_back(app, &settings, &mode, &transcript).await
    } else {
        transcript.clone()
    };

    // Hide the overlay before pasting: it must not be frontmost when the keystroke
    // lands, or the text goes to us instead of the app the user was typing in.
    overlay::hide(app);
    state::emit_status(app, Phase::Pasting, mode_id);

    // Without Accessibility permission the paste keystroke is swallowed by the OS with
    // no error at all, so check first: copying and saying why beats appearing to work
    // while nothing arrives in the target app.
    let wants_paste = matches!(mode.output, OutputAction::Paste);
    let can_paste = inject::can_synthesize_input();
    // A take that began while one of our own windows was frontmost has nowhere to send
    // the text: pressing the paste shortcut would deliver it into ourselves, which from
    // the outside is indistinguishable from the paste silently failing. Copy and say so.
    let has_target = target_app.can_receive_paste();

    let delivered = if wants_paste && can_paste && has_target {
        // Hand focus back to where the text belongs. Needed whenever anything of ours
        // took focus during the take — most obviously the overlay's Stop button.
        crate::focus::restore_and_settle(target_app);
        inject::paste(app, &final_text)
    } else {
        inject::copy(app, &final_text)
    };

    match delivered {
        Ok(()) if wants_paste && !can_paste => {
            show_error(app, ErrorPayload::new(ErrorKind::NeedsAccessibility));
            feedback::play(feedback::Cue::Error, settings.audio.feedback_volume);
        }
        Ok(()) if wants_paste && !has_target => {
            show_error(app, ErrorPayload::new(ErrorKind::NoPasteTarget));
            feedback::play(feedback::Cue::Error, settings.audio.feedback_volume);
        }
        Ok(()) => feedback::play(feedback::Cue::Done, settings.audio.feedback_volume),
        Err(e) => {
            show_error(
                app,
                ErrorPayload::with_detail(ErrorKind::DeliveryFailed, format!("{e:#}")),
            );
            feedback::play(feedback::Cue::Error, settings.audio.feedback_volume);
        }
    }

    record_history(app, &settings, &mode, &transcript, &final_text, &pcm);
    state::emit_status(app, Phase::Idle, mode_id);
}

/// Runs the AI pass, falling back to the raw transcript on any failure. A cleanup
/// error should degrade the output, never discard it.
async fn refine_or_fall_back(
    app: &AppHandle,
    settings: &crate::settings::AppSettings,
    mode: &crate::settings::Mode,
    transcript: &str,
) -> String {
    let llm = settings.llm_for(mode);
    let api_key = defaults::preset(&llm.preset)
        .filter(|p| p.needs_key)
        .and_then(|p| secrets::get_key(p.secret_key));

    if defaults::preset(&llm.preset).map(|p| p.needs_key).unwrap_or(true) && api_key.is_none() {
        emit_error(
            app,
            ErrorPayload::with_detail(ErrorKind::NoLlmKey, llm.preset.clone()),
        );
        return transcript.to_string();
    }

    let request = RefineRequest {
        system_prompt: mode.prompt.clone(),
        transcript: transcript.to_string(),
        llm,
        api_key,
    };

    match refine::refine(request).await {
        Ok(text) if !text.trim().is_empty() => text,
        Ok(_) => {
            log::warn!("AI cleanup returned nothing; using the raw transcript");
            transcript.to_string()
        }
        Err(e) => {
            emit_error(
                app,
                ErrorPayload::with_detail(ErrorKind::CleanupFailed, format!("{e:#}")),
            );
            transcript.to_string()
        }
    }
}

fn record_history(
    app: &AppHandle,
    settings: &crate::settings::AppSettings,
    mode: &crate::settings::Mode,
    raw_text: &str,
    final_text: &str,
    pcm: &[i16],
) {
    if !settings.history.enabled {
        return;
    }

    let id = uuid::Uuid::new_v4().to_string();
    let audio_path = if settings.history.keep_audio && !pcm.is_empty() {
        history::audio_dir(app)
            .map(|dir| dir.join(format!("{id}.wav")))
            .and_then(|path| audio::write_wav(&path, pcm).map(|()| path))
            .map_err(|e| log::warn!("could not save recording: {e:#}"))
            .ok()
            .map(|path| path.to_string_lossy().to_string())
    } else {
        None
    };

    let entry = HistoryEntry {
        id,
        created_at: Utc::now(),
        mode_id: mode.id.clone(),
        mode_name: mode.name.clone(),
        raw_text: raw_text.to_string(),
        final_text: final_text.to_string(),
        duration_secs: audio::duration_secs(pcm),
        stt_provider: format!("{:?}", settings.stt.provider),
        audio_path,
    };

    match history::append(app, entry, &settings.history) {
        Ok(()) => {
            let _ = app.emit(events::HISTORY_CHANGED, ());
        }
        Err(e) => log::warn!("could not write history: {e:#}"),
    }
}

/// Shared failure path: surface the message, play the error cue, return to idle.
pub fn fail(app: &AppHandle, mode_id: &str, payload: ErrorPayload) {
    let settings = state::settings_snapshot(app);
    // Idle first: the overlay renders the error only once it is no longer showing a
    // take in progress, and `show_error` is what decides when the panel goes away.
    state::emit_status(app, Phase::Idle, mode_id);
    show_error(app, payload);
    feedback::play(feedback::Cue::Error, settings.audio.feedback_volume);
}
