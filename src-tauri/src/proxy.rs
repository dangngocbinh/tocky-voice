//! Outbound HTTP proxy for the speech-recognition websocket.
//!
//! Exists for one measured reason. On some routes the direct path to the provider
//! loses packets badly enough that TCP collapses its send window; the audio stream
//! then backs up, committed text arrives more than ten seconds behind the speech, and
//! the final result lands after [`crate::stt::DRAIN_TIMEOUT`] has already given up on
//! it — so the tail of the sentence is silently lost. Tunnelling through a
//! well-connected proxy sidesteps that leg of the route entirely.
//!
//! **Only the speech websocket goes through here.** `reqwest` — AI cleanup and
//! text-to-speech — already honours `HTTPS_PROXY`/`ALL_PROXY` on its own, and nothing
//! measured so far says those paths need the help. Widening the scope is a deliberate
//! decision, not an oversight to be fixed by reflex.
//!
//! The address lives in settings; the username/password pair lives in the credential
//! vault under `"proxy"` like every other secret, and is read at connect time rather
//! than cached, so saving a new one takes effect on the very next take.

use crate::settings::{secrets, ProxySettings};
use anyhow::{anyhow, bail, Context, Result};
use base64::Engine as _;
use std::sync::RwLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Vault account holding `username:password`. Shares the credential store with the
/// provider keys so the "never read a secret back across the IPC boundary" rule covers
/// the proxy password too.
pub const SECRET_ACCOUNT: &str = "proxy";

/// Ceiling on the CONNECT reply's header block. A proxy that answers with an endless
/// stream of headers is broken, and reading it forever would hang the take.
const MAX_HEAD_BYTES: usize = 8 * 1024;

/// A resolved proxy endpoint. Credentials are deliberately absent — see the module doc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proxy {
    pub host: String,
    pub port: u16,
}

static ACTIVE: RwLock<Option<Proxy>> = RwLock::new(None);

/// Points the speech websocket at `settings`' proxy, or back at the direct route when
/// it is switched off. Call at startup and after every settings save.
///
/// An address that will not parse logs and falls back to direct rather than failing the
/// save: a typo in this field must not be able to lock the user out of dictating.
pub fn configure(settings: &ProxySettings) {
    let resolved = match (settings.enabled, parse_address(&settings.url)) {
        (false, _) => None,
        (true, Ok((host, port))) => Some(Proxy { host, port }),
        (true, Err(e)) => {
            log::warn!("proxy is on but its address is unusable ({e:#}); using the direct route");
            None
        }
    };
    match ACTIVE.write() {
        Ok(mut slot) => *slot = resolved,
        Err(e) => log::warn!("could not update the proxy setting: {e}"),
    }
}

/// The proxy to tunnel through, or `None` to connect directly.
pub fn current() -> Option<Proxy> {
    ACTIVE.read().ok().and_then(|slot| slot.clone())
}

/// Splits `host:port`, tolerating an `http://` prefix and a trailing slash because
/// that is how a proxy address is usually copied out of a browser or a chat message.
///
/// A bracketed IPv6 literal (`[::1]:3128`) keeps its brackets off the host, since
/// that is the form `lookup_host` wants.
pub fn parse_address(raw: &str) -> Result<(String, u16)> {
    let address = raw
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    if address.is_empty() {
        bail!("no proxy address set");
    }
    // Reject `user:pass@host:port` rather than letting it through. `rsplit_once(':')`
    // below splits on the *last* colon, so the userinfo would silently become part of
    // the hostname: the address could never connect, and the password would be handed
    // to `lookup_host` as a DNS label and printed into the "connecting to the proxy at
    // …" error the overlay shows on screen. Checked before the split so a portless
    // paste (`user:pass@host`) cannot reach the message below either — and the message
    // deliberately does not echo what was typed.
    if address.contains('@') {
        bail!(
            "put the proxy address on its own here (host:port) — \
             the username and password go in the credentials field below"
        );
    }
    let (host, port) = address
        .rsplit_once(':')
        .context("a proxy address needs a port, for example 10.0.0.1:3128")?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() {
        bail!("a proxy address needs a host, for example 10.0.0.1:3128");
    }
    let port: u16 = port
        .parse()
        .with_context(|| format!("{port:?} is not a port number"))?;
    if port == 0 {
        bail!("0 is not a usable proxy port");
    }
    Ok((host.to_string(), port))
}

/// Turns an already-open connection to the proxy into a transparent pipe to
/// `host:port`, ready for the TLS handshake to run over the top.
///
/// Speaks HTTP/1.1 CONNECT by hand rather than pulling in an HTTP client: the
/// websocket path owns its own socket (see [`crate::stt`]'s Happy Eyeballs dialling),
/// and `reqwest` has no way to hand a tunnelled stream back out.
pub async fn tunnel(stream: &mut TcpStream, host: &str, port: u16) -> Result<()> {
    let target = format!("{host}:{port}");
    let mut request = format!("CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n");
    if let Some(auth) = credentials() {
        request.push_str(&format!("Proxy-Authorization: {auth}\r\n"));
    }
    request.push_str("Proxy-Connection: Keep-Alive\r\n\r\n");

    stream
        .write_all(request.as_bytes())
        .await
        .context("sending CONNECT to the proxy")?;
    stream.flush().await.context("flushing CONNECT to the proxy")?;

    let head = read_head(stream).await?;
    let status = status_code(&head)?;
    if !(200..300).contains(&status) {
        let reason = head.lines().next().unwrap_or_default().trim();
        // 407 is worth naming: it is the one failure the user can fix themselves, and
        // "proxy refused" alone sends people hunting through firewall rules instead.
        if status == 407 {
            bail!("the proxy rejected the username/password (407) — check the proxy credentials in Settings");
        }
        bail!("the proxy refused a connection to {target}: {reason}");
    }
    Ok(())
}

/// The `Proxy-Authorization` header value, or `None` when the proxy needs no login.
fn credentials() -> Option<String> {
    basic_auth(secrets::get_key(SECRET_ACCOUNT)?.as_str())
}

fn basic_auth(credentials: &str) -> Option<String> {
    let credentials = credentials.trim();
    if credentials.is_empty() {
        return None;
    }
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
    Some(format!("Basic {encoded}"))
}

/// Reads the CONNECT reply up to and including the blank line that ends its headers.
///
/// One byte at a time on purpose: every byte after that blank line is already the TLS
/// handshake, and a buffered read would swallow the start of it — which surfaces much
/// later as an inscrutable handshake failure rather than as anything to do with the
/// proxy.
async fn read_head(stream: &mut TcpStream) -> Result<String> {
    let mut head: Vec<u8> = Vec::with_capacity(128);
    let mut byte = [0u8; 1];
    loop {
        let read = stream
            .read(&mut byte)
            .await
            .context("reading the proxy's reply to CONNECT")?;
        if read == 0 {
            bail!("the proxy closed the connection without answering CONNECT");
        }
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") {
            return Ok(String::from_utf8_lossy(&head).into_owned());
        }
        if head.len() > MAX_HEAD_BYTES {
            bail!("the proxy's reply to CONNECT never ended its headers");
        }
    }
}

fn status_code(head: &str) -> Result<u16> {
    let status_line = head.lines().next().unwrap_or_default();
    let code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("the proxy answered CONNECT with {status_line:?}"))?;
    code.parse()
        .with_context(|| format!("the proxy answered CONNECT with {status_line:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_host_and_port() {
        assert_eq!(
            parse_address("192.0.2.10:3128").unwrap(),
            ("192.0.2.10".to_string(), 3128)
        );
    }

    /// Addresses here are RFC 5737 documentation ranges on purpose — a test is not the
    /// place to record whatever proxy somebody happened to be using.
    ///
    /// A proxy address is usually pasted out of a browser or a chat message, so the
    /// scheme and a trailing slash come along with it far more often than not.
    #[test]
    fn tolerates_a_scheme_and_a_trailing_slash() {
        assert_eq!(
            parse_address("http://proxy.local:8080/").unwrap(),
            ("proxy.local".to_string(), 8080)
        );
    }

    #[test]
    fn strips_the_brackets_from_an_ipv6_literal() {
        assert_eq!(parse_address("[::1]:3128").unwrap(), ("::1".to_string(), 3128));
    }

    #[test]
    fn rejects_addresses_that_would_silently_go_nowhere() {
        for bad in ["", "   ", "proxy.local", "proxy.local:", "proxy.local:0", ":3128"] {
            assert!(parse_address(bad).is_err(), "{bad:?} should not parse");
        }
    }

    /// Regression: `rsplit_once(':')` splits on the *last* colon, so a pasted
    /// `user:pass@host:port` used to parse as the hostname `"user:pass@host"`. That
    /// could never connect, and it put the password into a DNS lookup and into the
    /// "connecting to the proxy at …" error the overlay paints over whatever the user
    /// is looking at. Rejected outright now, and the refusal must not repeat it back.
    #[test]
    fn refuses_a_url_with_credentials_in_it_without_echoing_them() {
        for pasted in [
            "http://squid:hunter2@192.0.2.10:3128",
            "squid:hunter2@192.0.2.10:3128",
            "http://squid:hunter2@192.0.2.10",
            "squid@192.0.2.10:3128",
        ] {
            let message = parse_address(pasted).expect_err(&format!("{pasted:?} must be refused")).to_string();
            assert!(
                !message.contains("hunter2") && !message.contains("squid"),
                "the refusal for {pasted:?} repeated the credentials back: {message}"
            );
        }
    }

    #[test]
    fn an_empty_credential_means_an_open_proxy_not_an_empty_login() {
        assert_eq!(basic_auth(""), None);
        assert_eq!(basic_auth("   "), None);
        assert_eq!(basic_auth("user:pass"), Some("Basic dXNlcjpwYXNz".to_string()));
    }

    #[test]
    fn reads_the_status_out_of_a_connect_reply() {
        assert_eq!(status_code("HTTP/1.1 200 Connection established\r\n\r\n").unwrap(), 200);
        assert_eq!(status_code("HTTP/1.1 407 Proxy Authentication Required\r\n\r\n").unwrap(), 407);
        assert!(status_code("garbage\r\n\r\n").is_err());
    }
}
