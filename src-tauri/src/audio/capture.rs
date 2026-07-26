//! Microphone capture. The cpal `Stream` is not `Send` on every platform, so it is
//! built and owned by a dedicated thread that parks until asked to stop; audio leaves
//! that thread as already-resampled 16 kHz mono PCM over a channel.

use super::resample;
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

pub struct CaptureChunk {
    pub pcm16: Vec<i16>,
    /// Peak amplitude of this chunk (0..1) for the UI level meter.
    pub peak: f32,
}

/// Dropping this stops the capture thread.
pub struct CaptureHandle {
    stop: Arc<AtomicBool>,
}

impl CaptureHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devices| devices.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default()
}

fn pick_device(name: Option<&str>) -> Result<cpal::Device> {
    let host = cpal::default_host();
    if let Some(name) = name {
        if let Ok(mut devices) = host.input_devices() {
            if let Some(dev) = devices.find(|d| d.name().map(|n| n == name).unwrap_or(false)) {
                return Ok(dev);
            }
        }
        log::warn!("input device {name:?} not found; falling back to system default");
    }
    host.default_input_device()
        .ok_or_else(|| anyhow!("no microphone available"))
}

/// Spawns capture. Returns once the stream is running, so a failure to open the
/// microphone surfaces to the caller instead of dying silently on the thread.
pub fn start(
    device_name: Option<String>,
    tx: UnboundedSender<CaptureChunk>,
) -> Result<CaptureHandle> {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = stop.clone();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<()>>();

    std::thread::Builder::new()
        .name("fvt-audio-capture".into())
        .spawn(move || {
            match build_stream(device_name.as_deref(), tx) {
                Ok(stream) => {
                    let _ = ready_tx.send(Ok(()));
                    // Keep `stream` alive here: dropping it closes the device.
                    while !thread_stop.load(Ordering::Relaxed) {
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    drop(stream);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            }
        })
        .context("spawning audio capture thread")?;

    ready_rx
        .recv()
        .context("audio capture thread died during startup")??;
    Ok(CaptureHandle { stop })
}

fn build_stream(
    device_name: Option<&str>,
    tx: UnboundedSender<CaptureChunk>,
) -> Result<cpal::Stream> {
    let device = pick_device(device_name)?;
    let config = device
        .default_input_config()
        .context("reading default input config")?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();

    log::info!(
        "capturing from {:?} at {sample_rate} Hz, {channels} ch, {sample_format:?}",
        device.name().unwrap_or_default()
    );

    let on_error = |err| log::error!("audio input stream error: {err}");

    // One closure body shared by every sample format; the wrappers just widen to f32.
    let forward = move |samples: Vec<f32>| {
        let mono = resample::to_mono(&samples, channels);
        let peak = resample::peak_level(&mono);
        let resampled = resample::resample_linear(&mono, sample_rate, TARGET_SAMPLE_RATE);
        if resampled.is_empty() {
            return;
        }
        // A closed receiver just means the session ended; the thread will notice its stop flag.
        let _ = tx.send(CaptureChunk {
            pcm16: resample::f32_to_i16(&resampled),
            peak,
        });
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &_| forward(data.to_vec()),
            on_error,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &stream_config,
            move |data: &[i16], _: &_| {
                forward(data.iter().map(|s| *s as f32 / i16::MAX as f32).collect())
            },
            on_error,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &stream_config,
            move |data: &[u16], _: &_| {
                forward(
                    data.iter()
                        .map(|s| (*s as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0))
                        .collect(),
                )
            },
            on_error,
            None,
        ),
        other => return Err(anyhow!("unsupported sample format: {other:?}")),
    }
    .context("building input stream")?;

    stream.play().context("starting input stream")?;
    Ok(stream)
}
