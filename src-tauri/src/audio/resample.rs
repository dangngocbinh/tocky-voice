//! Convert whatever the microphone gives us into the 16 kHz mono signed-16-bit PCM
//! that every streaming STT provider expects.

/// Average interleaved channels down to one.
pub fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    samples
        .chunks_exact(ch)
        .map(|frame| frame.iter().sum::<f32>() / ch as f32)
        .collect()
}

/// Linear-interpolating resampler. Speech at 16 kHz is forgiving enough that the
/// quality difference against a windowed-sinc filter is inaudible to the recognizer,
/// and this keeps the hot path allocation-light and dependency-free.
pub fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if input.is_empty() || from_rate == to_rate {
        return input.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = ((input.len() as f64) / ratio).floor() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = (pos - idx as f64) as f32;
        let a = input[idx];
        let b = *input.get(idx + 1).unwrap_or(&a);
        out.push(a + (b - a) * frac);
    }
    out
}

pub fn f32_to_i16(input: &[f32]) -> Vec<i16> {
    input
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect()
}

/// Little-endian byte view, which is the `pcm_s16le` / `linear16` encoding the
/// providers are configured for.
pub fn i16_to_le_bytes(input: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() * 2);
    for s in input {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// Peak amplitude in 0..1, used to drive the level meter in the UI.
pub fn peak_level(input: &[f32]) -> f32 {
    input.iter().fold(0.0f32, |acc, s| acc.max(s.abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downmixes_stereo_to_mono() {
        let stereo = [1.0, 0.0, 0.5, 0.5];
        assert_eq!(to_mono(&stereo, 2), vec![0.5, 0.5]);
    }

    #[test]
    fn resamples_48k_to_16k_by_a_third() {
        let input: Vec<f32> = (0..300).map(|i| i as f32 / 300.0).collect();
        assert_eq!(resample_linear(&input, 48_000, 16_000).len(), 100);
    }

    #[test]
    fn passes_through_when_rate_matches() {
        let input = vec![0.1, 0.2, 0.3];
        assert_eq!(resample_linear(&input, 16_000, 16_000), input);
    }
}
