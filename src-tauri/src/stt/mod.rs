//! Realtime speech-to-text over WebSocket.
//!
//! All three vendors follow the same shape — open a socket, optionally send a JSON
//! config frame, stream binary PCM, then send a finalize frame and drain the last
//! results. Only the URL, auth header, and result JSON differ, so that variation is
//! isolated behind [`WsProtocol`] and the transport lives here once.

pub mod assemblyai;
pub mod deepgram;
pub mod soniox;

use crate::audio::capture::TARGET_SAMPLE_RATE;
use crate::settings::{SttProviderKind, SttSettings};
use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;

/// How long to keep reading after we tell the provider we're done talking.
const DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);

/// How long to wait for the socket to open.
///
/// A blackholed route eventually errors at the OS level, but a stalled TLS handshake,
/// a hung proxy, or a captive portal never does — and an unbounded connect means the
/// take records with no partials and only reveals itself as a hang at stop time.
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// How long any single write to the socket may take.
///
/// Sends stall rather than fail when the far end stops reading and the kernel buffer
/// fills, which is what a half-dead connection looks like. This is awaited inside the
/// same `select!` that reads results, so a stalled write freezes the whole take.
const SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Smallest audio frame worth putting on the wire, in bytes of 16 kHz mono PCM16.
///
/// The microphone hands us whatever the device's buffer size works out to — often 10 ms
/// or less — and AssemblyAI rejects anything under 50 ms outright (error 3007, "Input
/// Duration Violation"), killing the socket a few seconds into a take. 100 ms sits in
/// the middle of every vendor's accepted range and cuts the frame rate tenfold, which
/// the other two providers are happier with too.
const MIN_FRAME_BYTES: usize = (TARGET_SAMPLE_RATE as usize / 10) * 2;

#[derive(Debug, Clone)]
pub enum SttEvent {
    /// Interim hypothesis — shown live, replaced by later text.
    Partial(String),
    /// Committed text, appended to the running transcript.
    Final(String),
}

/// Vendor-specific pieces of the streaming protocol.
pub trait WsProtocol: Send {
    /// Connection request, including any auth headers.
    fn request(&self) -> Result<Request<()>>;
    /// Configuration frame sent before any audio, if the vendor needs one.
    fn init_message(&self) -> Option<Message>;
    /// Frame that tells the vendor no more audio is coming.
    fn finish_message(&self) -> Message;
    /// Turn one inbound text frame into zero or more transcript events.
    ///
    /// Returns `Err` when the frame is the vendor telling us the session is over —
    /// a rejected key, an unknown model, an exhausted quota. Those arrive as ordinary
    /// text frames a second or two into the take and are followed by a close, so a
    /// parser that shrugs them off turns a fixable configuration mistake into an
    /// empty transcript and an overlay that vanishes for no stated reason.
    fn parse(&mut self, text: &str) -> Result<Vec<SttEvent>>;
}

pub fn build_protocol(settings: &SttSettings, api_key: String) -> Box<dyn WsProtocol> {
    match settings.provider {
        SttProviderKind::Soniox => Box::new(soniox::Soniox::new(settings, api_key)),
        SttProviderKind::Deepgram => Box::new(deepgram::Deepgram::new(settings, api_key)),
        SttProviderKind::AssemblyAi => Box::new(assemblyai::AssemblyAi::new(api_key)),
    }
}

/// Streams `audio_rx` to the provider until the sender is dropped, forwarding
/// interim results to `events`. Resolves with the complete final transcript.
pub async fn run_stream(
    mut protocol: Box<dyn WsProtocol>,
    mut audio_rx: UnboundedReceiver<Vec<u8>>,
    events: UnboundedSender<SttEvent>,
) -> Result<String> {
    let request = protocol.request()?;
    let (ws, _) = tokio::time::timeout(CONNECT_TIMEOUT, tokio_tungstenite::connect_async(request))
        .await
        .map_err(|_| {
            anyhow!(
                "the speech provider did not answer within {}s — check your connection",
                CONNECT_TIMEOUT.as_secs()
            )
        })?
        .context("connecting to speech provider")?;
    let (mut writer, mut reader) = ws.split();

    if let Some(init) = protocol.init_message() {
        writer.send(init).await.context("sending stt config")?;
    }

    let mut transcript = String::new();
    let mut audio_done = false;
    let mut pending = Vec::with_capacity(MIN_FRAME_BYTES * 2);
    // Why the socket stopped accepting audio, when it was not because the user
    // finished speaking. Reported at the end, but only if nothing was transcribed —
    // a stream that dies after producing words should still hand those words over.
    let mut broke_early: Option<String> = None;

    loop {
        tokio::select! {
            // Audio side: forward chunks; a closed channel means the take is over.
            chunk = audio_rx.recv(), if !audio_done => match chunk {
                Some(bytes) => {
                    pending.extend_from_slice(&bytes);
                    if pending.len() >= MIN_FRAME_BYTES {
                        let frame = std::mem::take(&mut pending);
                        pending.reserve(MIN_FRAME_BYTES * 2);
                        if let Err(e) = send_bounded(&mut writer, Message::Binary(frame)).await {
                            log::warn!("stt socket unusable while sending audio: {e}");
                            broke_early = Some(format!("the connection dropped mid-take: {e}"));
                            audio_done = true;
                        }
                    }
                }
                None => {
                    audio_done = true;
                    // The tail is usually shorter than a full frame. Vendors accept a
                    // short final frame; dropping it would clip the last syllable.
                    if !pending.is_empty() {
                        let tail = Message::Binary(std::mem::take(&mut pending));
                        let _ = send_bounded(&mut writer, tail).await;
                    }
                    let _ = send_bounded(&mut writer, protocol.finish_message()).await;
                }
            },

            // Result side.
            msg = reader.next() => match msg {
                Some(Ok(Message::Text(text))) => {
                    for event in protocol.parse(&text)? {
                        if let SttEvent::Final(ref t) = event {
                            append_segment(&mut transcript, t);
                        }
                        let _ = events.send(event);
                    }
                }
                // A close before the user has finished speaking is the provider ending
                // the session on us, and its reason is the only clue about why.
                Some(Ok(Message::Close(frame))) => {
                    if !audio_done {
                        broke_early = Some(close_reason(frame.as_ref()));
                    }
                    break;
                }
                None => {
                    if !audio_done {
                        broke_early
                            .get_or_insert_with(|| "the provider dropped the connection".into());
                    }
                    break;
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(anyhow!("speech provider stream error: {e}")),
            },
        }

        // Once audio has stopped, give the provider a bounded window to flush.
        if audio_done {
            match tokio::time::timeout(DRAIN_TIMEOUT, drain(&mut reader, &mut protocol, &events)).await
            {
                Ok(Ok(tail)) => {
                    for segment in tail {
                        append_segment(&mut transcript, &segment);
                    }
                }
                Ok(Err(e)) => {
                    log::warn!("stt drain error: {e}");
                    broke_early.get_or_insert_with(|| format!("{e}"));
                }
                Err(_) => log::warn!("stt drain timed out; using transcript so far"),
            }
            break;
        }
    }

    let _ = tokio::time::timeout(SEND_TIMEOUT, writer.close()).await;
    let transcript = transcript.trim().to_string();
    match broke_early {
        Some(reason) if transcript.is_empty() => Err(anyhow!("{reason}")),
        _ => Ok(transcript),
    }
}

/// Longest a key check is allowed to take. Every vendor answers a bad credential
/// within a second or two; this only has to stop a hung socket from hanging the UI.
const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// Opens a real stream, pushes a moment of silence through it, and closes.
///
/// Saving a speech key used to be a write with nothing on the other end, so a typo, a
/// revoked key or an out-of-credit account looked identical to success and only showed
/// up as a dictation that produced nothing. This runs the same path a real take does —
/// connect, authenticate, send the config frame, send PCM — so whatever the vendor
/// objects to is reported in its own words, at the moment the key is entered.
pub async fn probe(settings: &SttSettings, api_key: String) -> Result<()> {
    let protocol = build_protocol(settings, api_key);
    let (audio_tx, audio_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    // Held, not dropped: `run_stream` sends interim results here and there is nothing
    // to display them, but a closed receiver would be a different code path than the
    // one a real take takes.
    let (event_tx, _events) = mpsc::unbounded_channel::<SttEvent>();

    // Vendors that authenticate lazily only complain once audio arrives, so send some.
    // 200 ms of silence clears the 50 ms floor with room to spare.
    let silence = vec![0u8; (TARGET_SAMPLE_RATE as usize / 5) * 2];
    let _ = audio_tx.send(silence);
    drop(audio_tx);

    tokio::time::timeout(PROBE_TIMEOUT, run_stream(protocol, audio_rx, event_tx))
        .await
        .map_err(|_| anyhow!("the provider did not answer within {PROBE_TIMEOUT:?}"))??;
    Ok(())
}

/// Human-readable version of a WebSocket close frame. Vendors put the actual
/// complaint — "invalid api key", "unknown model" — in the reason string.
fn close_reason(frame: Option<&tokio_tungstenite::tungstenite::protocol::CloseFrame>) -> String {
    match frame {
        Some(frame) if !frame.reason.is_empty() => {
            format!("the provider closed the stream: {} ({})", frame.reason, frame.code)
        }
        Some(frame) => format!("the provider closed the stream (code {})", frame.code),
        None => "the provider closed the stream without saying why".into(),
    }
}

/// Writes one frame, treating "the socket never accepted it" the same as a write error.
///
/// A TCP connection whose far end has gone away without a FIN accepts data until the
/// send buffer fills and then simply stops — the await never resolves either way. That
/// is indistinguishable from a broken socket as far as the take is concerned, so it is
/// reported as one instead of stalling the loop that reads results.
async fn send_bounded<S>(writer: &mut S, message: Message) -> Result<()>
where
    S: futures_util::Sink<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::fmt::Display,
{
    match tokio::time::timeout(SEND_TIMEOUT, writer.send(message)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(anyhow!("{e}")),
        Err(_) => Err(anyhow!(
            "the socket stopped accepting audio for {}s",
            SEND_TIMEOUT.as_secs()
        )),
    }
}

#[cfg(test)]
mod send_tests {
    use super::*;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    /// What a half-dead TCP connection looks like once the send buffer fills: it never
    /// accepts the frame and never reports an error either.
    struct StalledSink;

    impl futures_util::Sink<Message> for StalledSink {
        type Error = std::io::Error;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }
        fn start_send(self: Pin<&mut Self>, _: Message) -> Result<(), Self::Error> {
            Ok(())
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }
        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Pending
        }
    }

    /// Regression: an unbounded send sits in the same `select!` as the result reader,
    /// so a stalled write froze the entire take with the panel left mid-dictation.
    #[tokio::test(start_paused = true)]
    async fn a_socket_that_never_accepts_a_frame_fails_instead_of_hanging() {
        let result = send_bounded(&mut StalledSink, Message::Binary(vec![0; MIN_FRAME_BYTES])).await;
        assert!(result.is_err(), "a stalled write has to be reported, not awaited");
    }
}

type WsReader = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

/// Reads until the provider closes the socket, collecting any remaining final text.
async fn drain(
    reader: &mut WsReader,
    protocol: &mut Box<dyn WsProtocol>,
    events: &UnboundedSender<SttEvent>,
) -> Result<Vec<String>> {
    let mut tail = Vec::new();
    while let Some(msg) = reader.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                for event in protocol.parse(&text)? {
                    if let SttEvent::Final(ref t) = event {
                        tail.push(t.clone());
                    }
                    let _ = events.send(event);
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(e) => return Err(anyhow!("{e}")),
        }
    }
    Ok(tail)
}

/// Joins transcript segments with exactly one space between words.
///
/// Vendors are inconsistent about whether a segment carries its own leading space —
/// Soniox usually does, Deepgram never does — so the segment is normalised and the
/// separator decided here rather than trusting either convention.
fn append_segment(transcript: &mut String, segment: &str) {
    let segment = segment.trim();
    if segment.is_empty() {
        return;
    }
    let attaches_to_previous_word = segment.starts_with(|c: char| ",.!?;:".contains(c));
    if !transcript.is_empty() && !transcript.ends_with(' ') && !attaches_to_previous_word {
        transcript.push(' ');
    }
    transcript.push_str(segment);
}

/// Shared helper for vendors that authenticate with a plain header.
pub(crate) fn request_with_header(url: &str, name: &'static str, value: &str) -> Result<Request<()>> {
    let mut request = url
        .into_client_request()
        .with_context(|| format!("building websocket request for {url}"))?;
    request.headers_mut().insert(
        name,
        value.parse().context("invalid auth header value")?,
    );
    Ok(request)
}

#[cfg(test)]
mod frame_tests {
    use super::{MIN_FRAME_BYTES, TARGET_SAMPLE_RATE};

    /// AssemblyAI closes the socket with error 3007 below 50 ms and above 1000 ms.
    /// Frames are flushed as soon as they reach the minimum, so the ceiling only has to
    /// hold for one accumulated frame plus the microphone chunk that tipped it over.
    #[test]
    fn frame_size_sits_inside_every_vendors_accepted_range() {
        let ms = |bytes: usize| bytes as f64 / (TARGET_SAMPLE_RATE as f64 * 2.0) * 1000.0;
        assert!(ms(MIN_FRAME_BYTES) >= 50.0, "{} ms is too short", ms(MIN_FRAME_BYTES));
        // Worst case: a full frame that was one byte short, plus a generous 100 ms chunk.
        assert!(ms(MIN_FRAME_BYTES * 2) <= 1000.0);
    }
}

#[cfg(test)]
mod tests {
    use super::append_segment;

    #[test]
    fn joins_segments_with_a_single_space() {
        let mut t = String::new();
        append_segment(&mut t, "xin chào");
        append_segment(&mut t, "mọi người");
        assert_eq!(t, "xin chào mọi người");
    }

    #[test]
    fn does_not_space_before_punctuation() {
        let mut t = String::from("xin chào");
        append_segment(&mut t, ", bạn khỏe không");
        assert_eq!(t, "xin chào, bạn khỏe không");
    }

    #[test]
    fn normalises_a_vendor_supplied_leading_space_to_exactly_one() {
        let mut t = String::from("hello");
        append_segment(&mut t, " world");
        assert_eq!(t, "hello world");
    }

    /// Regression: a segment arriving with a leading space used to have that space
    /// stripped without a replacement being inserted, producing "tôisẽ".
    #[test]
    fn keeps_words_separated_across_segment_boundaries() {
        let mut t = String::new();
        append_segment(&mut t, "Xin chào, hôm nay tôi");
        append_segment(&mut t, " sẽ deploy cái API này");
        assert_eq!(t, "Xin chào, hôm nay tôi sẽ deploy cái API này");
    }
}
