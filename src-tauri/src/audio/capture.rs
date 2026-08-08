//! Microphone capture. The cpal `Stream` is not `Send` on every platform, so it is
//! built and owned by a dedicated thread that parks until asked to stop; audio leaves
//! that thread as already-resampled 16 kHz mono PCM over a channel.

use super::resample;
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample as _;
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

/// One line per input, describing what the driver says it can do.
///
/// "That device does not work" is unanswerable from the outside: the sample format, the
/// rate and the channel count all come from the driver, they differ per device, and a
/// USB interface reports something quite unlike a built-in microphone. This is the
/// inventory to ask for in a bug report.
pub fn describe_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    let default = host
        .default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let Ok(devices) = host.input_devices() else {
        return vec!["could not enumerate input devices".into()];
    };

    devices
        .map(|device| {
            let name = device.name().unwrap_or_else(|_| "<unnamed>".into());
            let marker = if name == default { " (system default)" } else { "" };
            match device.default_input_config() {
                Ok(config) => {
                    let supported: Vec<String> = device
                        .supported_input_configs()
                        .map(|configs| {
                            configs
                                .map(|c| {
                                    format!(
                                        "{:?}/{}ch/{}-{}Hz",
                                        c.sample_format(),
                                        c.channels(),
                                        c.min_sample_rate().0,
                                        c.max_sample_rate().0
                                    )
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    format!(
                        "{name}{marker}: default {:?}/{}ch/{}Hz; supports [{}]",
                        config.sample_format(),
                        config.channels(),
                        config.sample_rate().0,
                        supported.join(", ")
                    )
                }
                Err(e) => format!("{name}{marker}: no usable input config ({e})"),
            }
        })
        .collect()
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

    // Bounded, because this is awaited on the thread that runs the window and the
    // hotkeys. Opening a device goes through the OS audio stack, and a wedged driver,
    // an exclusive-mode device, or a Bluetooth headset renegotiating its profile can
    // sit there indefinitely — which stops being "the microphone is slow" and becomes
    // "the whole app is frozen". Five seconds is far longer than any healthy open.
    match ready_rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(result) => result?,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            // The thread is left to finish opening and shut itself down: `stop` is
            // already shared with it, so setting it here is what ends it.
            stop.store(true, Ordering::Relaxed);
            return Err(anyhow!("the microphone did not open within 5s"));
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            return Err(anyhow!("audio capture thread died during startup"))
        }
    }
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

    // Every sample format cpal can hand us, converted to f32 by the same rule.
    //
    // Only F32, I16 and U16 used to be accepted and everything else was a hard error.
    // That is fine for a laptop's built-in microphone, which is almost always one of
    // those — but USB interfaces and "Line in" codecs commonly negotiate I32 (24 bits
    // carried in 32) under WASAPI shared mode, and those devices could not be used at
    // all. The conversion is `dasp`'s, so each format is scaled by its own range rather
    // than by a hand-written constant per arm.
    macro_rules! input_stream {
        ($sample:ty) => {
            device.build_input_stream(
                &stream_config,
                move |data: &[$sample], _: &_| {
                    forward(data.iter().map(|s| s.to_sample::<f32>()).collect())
                },
                on_error,
                None,
            )
        };
    }

    let stream = match sample_format {
        cpal::SampleFormat::F32 => input_stream!(f32),
        cpal::SampleFormat::F64 => input_stream!(f64),
        cpal::SampleFormat::I8 => input_stream!(i8),
        cpal::SampleFormat::I16 => input_stream!(i16),
        cpal::SampleFormat::I32 => input_stream!(i32),
        cpal::SampleFormat::I64 => input_stream!(i64),
        cpal::SampleFormat::U8 => input_stream!(u8),
        cpal::SampleFormat::U16 => input_stream!(u16),
        cpal::SampleFormat::U32 => input_stream!(u32),
        cpal::SampleFormat::U64 => input_stream!(u64),
        other => {
            return Err(anyhow!(
                "this input delivers {other:?} samples, which cpal does not give us a \
                 typed callback for. Inputs on this machine: [{}]",
                describe_input_devices().join(" | ")
            ))
        }
    }
    .context("building input stream")?;

    stream.play().context("starting input stream")?;
    Ok(stream)
}
