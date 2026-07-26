//! Short synthesized tones so dictation is usable without watching the screen.
//! Tones are generated rather than shipped as assets — no files to bundle, and the
//! pitch pattern alone tells you which event fired.

use rodio::source::{SineWave, Source};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    /// Recording began.
    Start,
    /// Recording ended, transcription in flight.
    Stop,
    /// Text was pasted successfully.
    Done,
    /// Take was discarded.
    Cancel,
    Error,
}

impl Cue {
    /// (frequency Hz, duration ms) pairs played back to back.
    fn tones(self) -> &'static [(f32, u64)] {
        match self {
            Cue::Start => &[(880.0, 70)],
            Cue::Stop => &[(660.0, 70)],
            Cue::Done => &[(880.0, 60), (1174.0, 90)],
            Cue::Cancel => &[(440.0, 60), (330.0, 90)],
            Cue::Error => &[(320.0, 130), (240.0, 160)],
        }
    }
}

/// Fire-and-forget. Playback happens on its own thread because the rodio
/// `OutputStream` must outlive the sink, and we never want to block dictation on audio.
pub fn play(cue: Cue, volume: f32) {
    let volume = volume.clamp(0.0, 1.0);
    if volume <= f32::EPSILON {
        return;
    }
    std::thread::spawn(move || {
        if let Err(e) = play_blocking(cue, volume) {
            log::debug!("feedback tone failed: {e}");
        }
    });
}

fn play_blocking(cue: Cue, volume: f32) -> anyhow::Result<()> {
    let (_stream, handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&handle)?;
    for (freq, ms) in cue.tones() {
        sink.append(
            SineWave::new(*freq)
                .take_duration(Duration::from_millis(*ms))
                .amplify(volume)
                // Fade in to avoid the click a hard-started sine produces.
                .fade_in(Duration::from_millis(8)),
        );
    }
    sink.sleep_until_end();
    Ok(())
}
