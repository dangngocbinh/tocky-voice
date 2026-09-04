//! Owns audio playback for a read-aloud session: a dedicated thread holding the
//! `rodio` output stream, fed a queue of PCM chunks and controlled by command.
//!
//! `rodio::OutputStream` is not `Send`, so it cannot be stored in `Player` itself and
//! moved across an `await` the way the rest of this app is written — the same
//! constraint `audio/feedback.rs` works around by giving playback its own thread. This
//! goes one step further and keeps that thread alive for the whole session so chunks
//! can be enqueued while earlier ones are still playing.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use super::Audio;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub enum Command {
    Enqueue(Audio),
    Pause,
    Resume,
    /// Clears the queue and silences immediately — does not finish the chunk in flight.
    Stop,
    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub played_ms: u64,
    pub total_ms: u64,
    pub paused: bool,
}

/// Progress bookkeeping shared between the caller and the playback thread.
///
/// Rodio's `Sink` exposes no "how far into the queue are we" query, and the plan here
/// deliberately does not try to reconstruct one from sample counts — a wall-clock timer
/// that stops on pause is accurate enough for a progress bar and immune to whatever
/// buffering `Sink` does internally.
#[derive(Default)]
struct Shared {
    total_ms: AtomicU64,
    /// Milliseconds accounted for up to the last pause/resume/stop boundary.
    accumulated_ms: AtomicU64,
    /// Set while actively playing; read to compute the live elapsed time on top of
    /// `accumulated_ms`. `None` while paused, stopped, or before the first chunk.
    playing_since: Mutex<Option<Instant>>,
    paused: AtomicBool,
}

impl Shared {
    fn reset(&self) {
        self.total_ms.store(0, Ordering::Relaxed);
        self.accumulated_ms.store(0, Ordering::Relaxed);
        *self.playing_since.lock().expect("player progress lock") = None;
        self.paused.store(false, Ordering::Relaxed);
    }

    fn mark_playing(&self) {
        let mut since = self.playing_since.lock().expect("player progress lock");
        if since.is_none() {
            *since = Some(Instant::now());
        }
        self.paused.store(false, Ordering::Relaxed);
    }

    fn mark_paused(&self) {
        self.settle();
        self.paused.store(true, Ordering::Relaxed);
    }

    /// Folds any time spent playing since the last boundary into `accumulated_ms` and
    /// clears the running clock, so the next read is exact instead of interpolated.
    fn settle(&self) {
        let mut since = self.playing_since.lock().expect("player progress lock");
        if let Some(start) = since.take() {
            let elapsed = start.elapsed().as_millis() as u64;
            self.accumulated_ms.fetch_add(elapsed, Ordering::Relaxed);
        }
    }

    fn progress(&self) -> Progress {
        let base = self.accumulated_ms.load(Ordering::Relaxed);
        let live = self
            .playing_since
            .lock()
            .expect("player progress lock")
            .map(|start| start.elapsed().as_millis() as u64)
            .unwrap_or(0);
        Progress {
            played_ms: base + live,
            total_ms: self.total_ms.load(Ordering::Relaxed),
            paused: self.paused.load(Ordering::Relaxed),
        }
    }
}

#[derive(Default)]
pub struct Player {
    tx: Mutex<Option<Sender<Command>>>,
    shared: Arc<Shared>,
}

impl Player {
    /// Spawns the thread that owns the `OutputStream`. Safe to call again after a prior
    /// session ended — a dead thread just leaves `tx` empty, and this rebuilds it.
    pub fn start(&self) -> anyhow::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel::<Command>();
        let shared = self.shared.clone();
        shared.reset();

        std::thread::Builder::new()
            .name("tts-player".into())
            .spawn(move || run(rx, shared))
            .map_err(|e| anyhow::anyhow!("could not start the playback thread: {e}"))?;

        *self.tx.lock().expect("player tx lock") = Some(tx);
        Ok(())
    }

    /// Sends a command if a session is running; a no-op otherwise, so callers never
    /// have to check `is_playing` first just to avoid an error nobody would show.
    pub fn send(&self, cmd: Command) {
        let guard = self.tx.lock().expect("player tx lock");
        if let Some(tx) = guard.as_ref() {
            if tx.send(cmd).is_err() {
                log::warn!("tts playback thread is gone; dropping command");
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        self.tx.lock().expect("player tx lock").is_some() && !self.shared.paused.load(Ordering::Relaxed)
    }

    pub fn progress(&self) -> Progress {
        self.shared.progress()
    }
}

/// The playback thread body: owns the stream and sink for the whole session and blocks
/// on the command channel between chunks.
fn run(rx: Receiver<Command>, shared: Arc<Shared>) {
    let (_stream, handle) = match rodio::OutputStream::try_default() {
        Ok(s) => s,
        Err(e) => {
            log::error!("could not open an audio output for read-aloud: {e}");
            return;
        }
    };
    let mut sink = match rodio::Sink::try_new(&handle) {
        Ok(s) => s,
        Err(e) => {
            log::error!("could not create the read-aloud playback sink: {e}");
            return;
        }
    };

    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::Enqueue(audio) => {
                let duration_ms = if audio.sample_rate > 0 {
                    (audio.pcm.len() as u64 * 1000) / audio.sample_rate as u64
                } else {
                    0
                };
                shared.total_ms.fetch_add(duration_ms, Ordering::Relaxed);
                sink.append(rodio::buffer::SamplesBuffer::new(1, audio.sample_rate, audio.pcm));
                if !sink.is_paused() {
                    shared.mark_playing();
                }
            }
            Command::Pause => {
                sink.pause();
                shared.mark_paused();
            }
            Command::Resume => {
                sink.play();
                shared.mark_playing();
            }
            Command::Stop => {
                // A fresh sink rather than trying to clear the existing one — the
                // simplest way to guarantee silence within the <100ms budget without
                // depending on exactly what `Sink::stop` leaves reusable.
                sink.stop();
                sink = match rodio::Sink::try_new(&handle) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("could not rebuild the playback sink after stop: {e}");
                        return;
                    }
                };
                shared.reset();
            }
            Command::Shutdown => {
                sink.stop();
                return;
            }
        }
    }
}

/// Rough playback duration of a chunk, used by callers that want to reason about
/// timing before a chunk has actually been enqueued (e.g. deciding whether the queue
/// is deep enough).
pub fn duration_ms(audio: &Audio) -> u64 {
    if audio.sample_rate == 0 {
        return 0;
    }
    (audio.pcm.len() as u64 * 1000) / audio.sample_rate as u64
}
