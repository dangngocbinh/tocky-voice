//! Failures the user is shown.
//!
//! The backend sends a *kind*, not a sentence. Every string the app writes itself lives
//! in the frontend dictionary, so a message composed in Rust would arrive in English no
//! matter which language the user picked — which is exactly what happened before this
//! existed.
//!
//! `detail` carries the technical cause when there is one, and stays untranslated on
//! purpose: it is the vendor's or the OS's wording, and mangling it would make the one
//! searchable part of an error report useless.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// macOS will not let us synthesize the paste keystroke.
    NeedsAccessibility,
    /// The take began while one of our own windows was frontmost.
    NoPasteTarget,
    /// Clipboard or keystroke delivery failed outright.
    DeliveryFailed,
    /// No credential saved for the selected speech provider.
    NoSttKey,
    /// No credential saved for the selected AI provider; raw transcript used instead.
    NoLlmKey,
    /// The AI pass failed; raw transcript used instead.
    CleanupFailed,
    /// The microphone could not be opened.
    MicUnavailable,
    /// The microphone opened but delivered nothing but silence for the whole take.
    NoAudioCaptured,
    /// The speech stream failed before producing a transcript.
    TranscriptionFailed,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub kind: ErrorKind,
    /// Underlying cause, verbatim. `None` when the kind says everything.
    pub detail: Option<String>,
}

impl ErrorPayload {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, detail: None }
    }

    pub fn with_detail(kind: ErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: Some(detail.into()),
        }
    }
}
