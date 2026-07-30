//! Live round-trip against the real speech providers.
//!
//! These exercise the actual WebSocket protocol implementations — the part most likely
//! to break when a vendor changes their wire format, and the part unit tests can't cover.
//! They need network access and a real API key, so they are `#[ignore]`d by default:
//!
//! ```sh
//! FVT_TEST_WAV=/path/to/16k-mono.wav \
//! SONIOX_API_KEY=... DEEPGRAM_API_KEY=... ASSEMBLYAI_API_KEY=... \
//! cargo test --test stt_provider_round_trip -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `--test-threads=1` is not optional: free tiers cap concurrent streaming sessions, so
//! running these in parallel fails a test for reasons that have nothing to do with the code.

use tockyvoice_lib::audio::capture::TARGET_SAMPLE_RATE;
use tockyvoice_lib::settings::{SttProviderKind, SttSettings};
use tockyvoice_lib::stt;
use tokio::sync::mpsc;

/// What live capture actually delivers: cpal hands over one device buffer at a time,
/// which on macOS is ~512 frames at 48 kHz — about 10 ms once resampled to 16 kHz.
///
/// This deliberately feeds frames that are *too small to send*. AssemblyAI rejects
/// anything under 50 ms and closes the socket, which used to kill a take a few seconds
/// in while the overlay carried on as if it were still listening. Streaming at a
/// realistic chunk size is what makes these tests able to catch that.
const CHUNK_SAMPLES: usize = 160; // 10 ms at 16 kHz
const CHUNK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

fn load_wav() -> Vec<i16> {
    let path = std::env::var("FVT_TEST_WAV").expect("set FVT_TEST_WAV to a 16 kHz mono WAV");
    let mut reader = hound::WavReader::open(&path).expect("opening test wav");
    let spec = reader.spec();
    assert_eq!(spec.channels, 1, "test wav must be mono");
    assert_eq!(
        spec.sample_rate, TARGET_SAMPLE_RATE,
        "test wav must be {TARGET_SAMPLE_RATE} Hz"
    );
    reader.samples::<i16>().map(|s| s.unwrap()).collect()
}

fn settings(provider: SttProviderKind) -> SttSettings {
    SttSettings {
        provider,
        soniox_model: "stt-rt-preview".into(),
        deepgram_model: "nova-2".into(),
        language: "vi".into(),
        language_hints: vec!["vi".into(), "en".into()],
    }
}

/// Streams the fixture through a provider and returns the final transcript.
async fn transcribe(provider: SttProviderKind, api_key: String) -> String {
    let pcm = load_wav();
    let settings = settings(provider);
    let protocol = stt::build_protocol(&settings, api_key);

    let (audio_tx, audio_rx) = mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    // Feed audio in real time. Sending the whole file at once would let the
    // provider's endpoint detection fire before it has seen the speech.
    tokio::spawn(async move {
        for chunk in pcm.chunks(CHUNK_SAMPLES) {
            let bytes = chunk.iter().flat_map(|s| s.to_le_bytes()).collect();
            if audio_tx.send(bytes).is_err() {
                return;
            }
            tokio::time::sleep(CHUNK_INTERVAL).await;
        }
        // Dropping the sender is what tells the stream the take is finished.
    });

    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            if let stt::SttEvent::Partial(text) = event {
                println!("  partial: {text}");
            }
        }
    });

    stt::run_stream(protocol, audio_rx, event_tx)
        .await
        .expect("stt stream failed")
}

/// A key the vendor rejects has to come back as an error, not as an empty transcript.
///
/// This is the regression behind "the recording panel appears for two seconds and then
/// closes itself": a rejected credential arrives as an ordinary message frame followed
/// by a close, and both used to be discarded, so the take ended with nothing to show
/// and nothing to say. Needs no credentials — a deliberately invalid one is the point —
/// so this is the one live test that runs without any account.
#[tokio::test]
#[ignore = "hits the live provider APIs"]
async fn a_rejected_key_is_reported_rather_than_swallowed() {
    for provider in [
        SttProviderKind::Soniox,
        SttProviderKind::Deepgram,
        SttProviderKind::AssemblyAi,
    ] {
        let result = stt::probe(&settings(provider.clone()), "not-a-real-key".into()).await;
        let err = result.expect_err(&format!("{provider:?} accepted an invalid key"));
        println!("{provider:?} rejected the key with: {err:#}");
        assert!(
            !format!("{err:#}").trim().is_empty(),
            "{provider:?} failed without saying why"
        );
    }
}

#[tokio::test]
#[ignore = "hits the live Soniox API"]
async fn soniox_transcribes_the_fixture() {
    let key = std::env::var("SONIOX_API_KEY").expect("set SONIOX_API_KEY");
    let transcript = transcribe(SttProviderKind::Soniox, key).await;
    println!("\nSoniox transcript: {transcript}\n");
    assert!(
        !transcript.trim().is_empty(),
        "Soniox returned an empty transcript"
    );
}

/// AssemblyAI's v3 stream stays silent until a *turn* ends, so a take that never pauses
/// produces no committed text at all. This asserts the transport survives a full take —
/// the failure being reproduced is the stream dying mid-recording, which the UI showed
/// only as a frozen level meter.
#[tokio::test]
#[ignore = "hits the live AssemblyAI API"]
async fn assemblyai_transcribes_the_fixture() {
    let key = std::env::var("ASSEMBLYAI_API_KEY").expect("set ASSEMBLYAI_API_KEY");
    let transcript = transcribe(SttProviderKind::AssemblyAi, key).await;
    println!("\nAssemblyAI transcript: {transcript}\n");
    assert!(
        !transcript.trim().is_empty(),
        "AssemblyAI returned an empty transcript"
    );
}

#[tokio::test]
#[ignore = "hits the live Deepgram API"]
async fn deepgram_transcribes_the_fixture() {
    let key = std::env::var("DEEPGRAM_API_KEY").expect("set DEEPGRAM_API_KEY");
    let transcript = transcribe(SttProviderKind::Deepgram, key).await;
    println!("\nDeepgram transcript: {transcript}\n");
    assert!(
        !transcript.trim().is_empty(),
        "Deepgram returned an empty transcript"
    );
}
