pub mod capture;
pub mod feedback;
pub mod mic_test;
pub mod resample;

use anyhow::{Context, Result};
use std::path::Path;

/// Write captured PCM to a 16 kHz mono WAV so a bad transcription can be replayed later.
pub fn write_wav(path: &Path, pcm16: &[i16]) -> Result<()> {
    if let Some(parent) = path.parent() {
        crate::private_file::create_dir(parent)?;
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: capture::TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).context("creating wav file")?;
    for sample in pcm16 {
        writer.write_sample(*sample)?;
    }
    writer.finalize().context("finalizing wav file")?;
    // A recording of the user's voice deserves the same protection as the transcript.
    crate::private_file::restrict_file(path);
    Ok(())
}

/// Seconds of audio represented by a PCM buffer at the capture sample rate.
pub fn duration_secs(pcm16: &[i16]) -> f32 {
    pcm16.len() as f32 / capture::TARGET_SAMPLE_RATE as f32
}
