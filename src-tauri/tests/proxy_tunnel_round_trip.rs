//! Live round-trip for the CONNECT tunnel, against a real proxy and a real provider.
//!
//! Unit tests cover parsing an address and reading a status line; they cannot tell you
//! whether a tunnelled socket actually carries a TLS handshake, a websocket upgrade and
//! a streaming take. Only this can, so it exists — `#[ignore]`d by default because it
//! needs network access, a proxy and a paid key:
//!
//! ```sh
//! FVT_TEST_WAV=/path/to/16k-mono.wav \
//! FVT_TEST_PROXY=10.0.0.1:3128 FVT_TEST_PROXY_CREDENTIALS=user:pass \
//! SONIOX_API_KEY=... \
//! cargo test --test proxy_tunnel_round_trip -- --ignored --nocapture
//! ```
//!
//! Leave `FVT_TEST_PROXY` unset to run the same take down the direct route instead,
//! which is how the two are compared: the number that matters is how long the provider
//! takes to finish *after* the audio stops, because `stt::DRAIN_TIMEOUT` is what gives
//! up on a slow one and drops the tail of the sentence.

use tockyvoice_lib::audio::capture::TARGET_SAMPLE_RATE;
use tockyvoice_lib::proxy;
use tockyvoice_lib::settings::{secrets, ProxySettings, SttProviderKind, SttSettings};
use tockyvoice_lib::stt;
use tokio::sync::mpsc;

const CHUNK_SAMPLES: usize = 1600; // 100 ms at 16 kHz, one frame as the app sends it
const CHUNK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// A throwaway credential vault holding just what this test needs, so it never reads —
/// or writes — the real one. Deleted on drop, and `0600` while it exists: it holds a
/// live API key and a proxy password, and the system temp directory is shared.
struct TempVault(std::path::PathBuf);

impl TempVault {
    fn with(entries: &[(&str, String)]) -> Self {
        let dir = std::env::temp_dir().join(format!("tockyvoice-test-vault-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp vault dir");
        let map: serde_json::Map<String, serde_json::Value> = entries
            .iter()
            .map(|(k, v)| ((*k).to_string(), serde_json::Value::String(v.clone())))
            .collect();
        let file = dir.join("credentials.json");
        std::fs::write(&file, serde_json::to_vec(&serde_json::Value::Object(map)).unwrap())
            .expect("writing temp vault");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600))
                .expect("locking down temp vault");
        }
        secrets::configure(dir.clone(), false);
        Self(dir)
    }
}

impl Drop for TempVault {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn load_wav() -> Vec<i16> {
    let path = std::env::var("FVT_TEST_WAV").expect("set FVT_TEST_WAV to a 16 kHz mono WAV");
    let mut reader = hound::WavReader::open(&path).expect("opening test wav");
    let spec = reader.spec();
    assert_eq!(spec.channels, 1, "test wav must be mono");
    assert_eq!(spec.sample_rate, TARGET_SAMPLE_RATE, "test wav must be 16 kHz");
    reader.samples::<i16>().map(|s| s.unwrap()).collect()
}

#[tokio::test]
#[ignore = "needs network, a proxy and a real key"]
async fn a_take_completes_through_the_tunnel() {
    let key = std::env::var("SONIOX_API_KEY").expect("set SONIOX_API_KEY");
    let address = std::env::var("FVT_TEST_PROXY").unwrap_or_default();
    let credentials = std::env::var("FVT_TEST_PROXY_CREDENTIALS").unwrap_or_default();

    let _vault = TempVault::with(&[
        ("soniox", key.clone()),
        (proxy::SECRET_ACCOUNT, credentials),
    ]);
    proxy::configure(&ProxySettings {
        enabled: !address.is_empty(),
        url: address.clone(),
    });
    let route = if address.is_empty() { "direct".to_string() } else { format!("proxy {address}") };
    assert_eq!(
        proxy::current().is_some(),
        !address.is_empty(),
        "the proxy setting did not take effect"
    );

    let settings = SttSettings {
        provider: SttProviderKind::Soniox,
        soniox_model: "stt-rt-preview".into(),
        deepgram_model: "nova-2".into(),
        language: "vi".into(),
        language_hints: vec!["vi".into(), "en".into()],
    };

    let samples = load_wav();
    let audio_seconds = samples.len() as f64 / TARGET_SAMPLE_RATE as f64;
    let (audio_tx, audio_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (event_tx, mut events) = mpsc::unbounded_channel::<stt::SttEvent>();

    let started = std::time::Instant::now();
    let first_token = std::sync::Arc::new(std::sync::Mutex::new(None::<f64>));
    let seen = first_token.clone();
    tokio::spawn(async move {
        while let Some(event) = events.recv().await {
            // Both variants carry the text; which one it is does not matter here, only
            // that something recognisable came back and when.
            let (stt::SttEvent::Partial(text) | stt::SttEvent::Final(text)) = &event;
            if !text.trim().is_empty() {
                seen.lock().unwrap().get_or_insert(started.elapsed().as_secs_f64());
            }
        }
    });

    let stream = tokio::spawn(stt::run_stream(
        stt::build_protocol(&settings, key),
        audio_rx,
        event_tx,
    ));

    // Real-time pacing, in the same 100 ms frames the microphone path produces. Sending
    // it all at once would hide the whole problem this proxy exists for: a backed-up
    // upload only shows itself when the audio arrives at the speed it is spoken.
    for chunk in samples.chunks(CHUNK_SAMPLES) {
        let bytes: Vec<u8> = chunk.iter().flat_map(|s| s.to_le_bytes()).collect();
        audio_tx.send(bytes).expect("stream ended early");
        tokio::time::sleep(CHUNK_INTERVAL).await;
    }
    drop(audio_tx);

    let audio_done = std::time::Instant::now();
    let outcome = stream.await.expect("stream task panicked").expect("stream failed");
    let finalize = audio_done.elapsed().as_secs_f64();

    println!(
        "[{route}] audio={audio_seconds:.1}s  first_token={}  finalize_after_audio={finalize:.2}s  incomplete={:?}",
        first_token
            .lock()
            .unwrap()
            .map(|t| format!("{t:.2}s"))
            .unwrap_or_else(|| "never".into()),
        outcome.incomplete,
    );
    println!("[{route}] transcript: {}", outcome.transcript);

    assert!(
        !outcome.transcript.trim().is_empty(),
        "the tunnel carried no transcript"
    );
    assert!(
        outcome.incomplete.is_none(),
        "the take came back incomplete: {:?}",
        outcome.incomplete
    );
}
