//! A local proxy gateway.
//!
//! Chrome's `--proxy-server` flag accepts `http://`/`socks5://` upstreams, but it
//! **cannot** carry `user:pass` credentials and cannot switch between proxies.
//! So instead of pointing Chrome at an upstream directly, we point it at a tiny
//! proxy we run ourselves on `127.0.0.1`:
//!
//! ```text
//!   Chrome  --(HTTP/CONNECT)-->  xcelerate gateway  --(HTTP, +Proxy-Authorization)-->  upstream pool
//! ```
//!
//! The gateway accepts the browser's `CONNECT host:port` (HTTPS) and absolute-form
//! (`GET http://host/...`, HTTP) requests, picks the next upstream from the pool
//! (round-robin), replays the request to it with a `Proxy-Authorization` header,
//! and copies bytes both ways. Because the gateway owns the upstream choice, a
//! pool, per-request rotation, and credential injection are all possible without
//! any Chrome support.
//!
//! Configuration is process-wide (a proxy is normally one setting per process):
//! call [`configure`] from Rust, or set `XCELERATE_PROXY_POOL` /
//! `XCELERATE_PROXY` (comma-separated upstream URLs) so every language binding
//! can use it through the environment.

use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine as _;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::{XcelerateError, XcelerateResult};

/// Largest request head the gateway will buffer from a client.
const HEAD_LIMIT: usize = 64 * 1024;

/// Per-connection ceiling for connect + head reads.
const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// One upstream HTTP proxy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Upstream {
    host: String,
    port: u16,
    /// `base64("user:pass")`, ready for a `Proxy-Authorization: Basic` header.
    auth: Option<String>,
}

/// Parse an upstream proxy URL.
///
/// Accepts `http://user:pass@host:port`, `http://host:port`, `host:port`, `host`
/// (port defaults to 80). `socks5://` is rejected with guidance, because Chrome
/// supports SOCKS natively via `--proxy-server` and does not need the gateway.
fn parse_upstream(url: &str) -> XcelerateResult<Upstream> {
    let url = url.trim();
    if url.is_empty() {
        return Err(XcelerateError::Unsupported("empty proxy URL".to_string()));
    }
    let (scheme, rest) = match url.split_once("://") {
        Some((scheme, rest)) => (scheme.to_ascii_lowercase(), rest),
        None => ("http".to_string(), url),
    };
    match scheme.as_str() {
        "http" => {}
        "https" => {
            return Err(XcelerateError::Unsupported(
                "https:// upstream proxies are not supported yet (TLS-to-proxy)".to_string(),
            ));
        }
        "socks5" | "socks4" | "socks" => {
            return Err(XcelerateError::Unsupported(
                "SOCKS upstreams are unnecessary: Chrome speaks SOCKS natively via --proxy-server"
                    .to_string(),
            ));
        }
        other => {
            return Err(XcelerateError::Unsupported(format!(
                "unsupported proxy scheme '{other}'"
            )));
        }
    }

    let (userinfo, authority) = match rest.rsplit_once('@') {
        Some((userinfo, authority)) => (Some(userinfo), authority),
        None => (None, rest),
    };
    let authority = authority.trim_end_matches('/');
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port: u16 = port
                .parse()
                .map_err(|_| XcelerateError::Unsupported(format!("bad proxy port in '{url}'")))?;
            (host.to_string(), port)
        }
        None => (authority.to_string(), 80),
    };
    if host.is_empty() {
        return Err(XcelerateError::Unsupported(format!(
            "missing proxy host in '{url}'"
        )));
    }
    let auth = userinfo
        .filter(|userinfo| !userinfo.is_empty())
        .map(|userinfo| base64::engine::general_purpose::STANDARD.encode(userinfo));
    Ok(Upstream { host, port, auth })
}

/// A round-robin pool of upstream proxies.
struct ProxyPool {
    upstreams: Vec<Upstream>,
    next: AtomicUsize,
}

impl ProxyPool {
    fn new(upstreams: Vec<Upstream>) -> Self {
        Self {
            upstreams,
            next: AtomicUsize::new(0),
        }
    }

    fn next_upstream(&self) -> Option<&Upstream> {
        if self.upstreams.is_empty() {
            return None;
        }
        let index = self.next.fetch_add(1, Ordering::SeqCst) % self.upstreams.len();
        self.upstreams.get(index)
    }
}

/// Process-wide proxy configuration, set by [`configure`] or the environment.
static POOL: Mutex<Option<Vec<Upstream>>> = Mutex::new(None);

/// Configure the upstream proxy pool for this process.
///
/// `urls` is a list of `http://[user:pass@]host:port` entries; the gateway
/// rotates through them. An empty list disables the gateway.
pub fn configure(urls: &[String]) -> XcelerateResult<()> {
    let mut parsed = Vec::with_capacity(urls.len());
    for url in urls {
        parsed.push(parse_upstream(url)?);
    }
    *POOL.lock().unwrap() = if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    };
    Ok(())
}

/// The configured upstreams, from [`configure`] or the environment.
fn configured_upstreams() -> XcelerateResult<Option<Vec<Upstream>>> {
    if let Some(upstreams) = POOL.lock().unwrap().clone() {
        return Ok(Some(upstreams));
    }
    for key in ["XCELERATE_PROXY_POOL", "XCELERATE_PROXY"] {
        if let Ok(value) = std::env::var(key) {
            let urls: Vec<String> = value
                .split(',')
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect();
            if !urls.is_empty() {
                let mut parsed = Vec::with_capacity(urls.len());
                for url in &urls {
                    parsed.push(parse_upstream(url)?);
                }
                return Ok(Some(parsed));
            }
        }
    }
    Ok(None)
}

/// A running gateway. Dropping it stops accepting connections.
pub(crate) struct ProxyGateway {
    addr: std::net::SocketAddr,
    handle: tokio::task::JoinHandle<()>,
}

impl ProxyGateway {
    /// Bind a gateway on `127.0.0.1:0` and start serving.
    async fn start(upstreams: Vec<Upstream>) -> XcelerateResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|error| XcelerateError::Unsupported(format!("proxy bind failed: {error}")))?;
        let addr = listener
            .local_addr()
            .map_err(|_| XcelerateError::InternalError)?;
        let pool = Arc::new(ProxyPool::new(upstreams));
        let handle = tokio::spawn(async move {
            loop {
                let (client, _) = match listener.accept().await {
                    Ok(connection) => connection,
                    Err(_) => break,
                };
                let pool = Arc::clone(&pool);
                tokio::spawn(async move {
                    let _ = tokio::time::timeout(IO_TIMEOUT, handle_client(client, pool)).await;
                });
            }
        });
        Ok(Self { addr, handle })
    }

    /// The URL to hand Chrome as `--proxy-server`.
    pub(crate) fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

impl Drop for ProxyGateway {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Start the gateway if a proxy pool is configured (programmatically or via env).
pub(crate) async fn start_if_configured() -> XcelerateResult<Option<ProxyGateway>> {
    match configured_upstreams()? {
        Some(upstreams) if !upstreams.is_empty() => Ok(Some(ProxyGateway::start(upstreams).await?)),
        _ => Ok(None),
    }
}

/// Read a request head (up to the terminating blank line), one byte at a time so
/// no body bytes are consumed from the socket.
async fn read_head(stream: &mut TcpStream) -> io::Result<String> {
    let mut buffer = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    loop {
        let read = stream.read(&mut byte).await?;
        if read == 0 {
            break;
        }
        buffer.push(byte[0]);
        if buffer.len() > HEAD_LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request head too large",
            ));
        }
        if buffer.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

/// A parsed request head: `(method, target, headers)`.
type RequestHead = (String, String, Vec<(String, String)>);

fn parse_head(head: &str) -> Option<RequestHead> {
    let mut lines = head.split("\r\n");
    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    Some((method, target, headers))
}

/// Handle one browser connection end to end.
async fn handle_client(mut client: TcpStream, pool: Arc<ProxyPool>) -> io::Result<()> {
    let head = read_head(&mut client).await?;
    let Some((method, target, headers)) = parse_head(&head) else {
        let _ = client.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
        return Ok(());
    };

    let Some(upstream) = pool.next_upstream() else {
        let _ = client
            .write_all(b"HTTP/1.1 502 No upstream proxy configured\r\n\r\n")
            .await;
        return Ok(());
    };

    let mut server = TcpStream::connect((upstream.host.as_str(), upstream.port)).await?;

    if method.eq_ignore_ascii_case("CONNECT") {
        // Replay the CONNECT to the upstream with our credentials.
        let mut request = format!("CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n");
        if let Some(auth) = &upstream.auth {
            request.push_str(&format!("Proxy-Authorization: Basic {auth}\r\n"));
        }
        request.push_str("\r\n");
        server.write_all(request.as_bytes()).await?;

        let upstream_head = read_head(&mut server).await?;
        let status = upstream_head
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("502");
        if !status.starts_with('2') {
            let _ = client
                .write_all(format!("HTTP/1.1 {}\r\n\r\n", status).as_bytes())
                .await;
            return Ok(());
        }
        client
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;
        tokio::io::copy_bidirectional(&mut client, &mut server).await?;
    } else {
        // Absolute-form request (plain HTTP through the proxy).
        let mut request = format!("{method} {target} HTTP/1.1\r\n");
        let mut has_auth = false;
        for (name, value) in &headers {
            if name.eq_ignore_ascii_case("proxy-connection") {
                continue;
            }
            if name.eq_ignore_ascii_case("proxy-authorization") {
                has_auth = true;
            }
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        if let Some(auth) = &upstream.auth
            && !has_auth
        {
            request.push_str(&format!("Proxy-Authorization: Basic {auth}\r\n"));
        }
        request.push_str("\r\n");
        server.write_all(request.as_bytes()).await?;
        tokio::io::copy_bidirectional(&mut client, &mut server).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    fn parse_ok(url: &str) -> Upstream {
        parse_upstream(url).unwrap()
    }

    #[test]
    fn parses_http_upstreams() {
        assert_eq!(
            parse_ok("http://127.0.0.1:8080"),
            Upstream {
                host: "127.0.0.1".into(),
                port: 8080,
                auth: None
            }
        );
        assert_eq!(parse_ok("host:3128").host, "host");
        assert_eq!(parse_ok("host").port, 80);

        let with_auth = parse_ok("http://user:secret@proxy.example:8080");
        assert_eq!(with_auth.host, "proxy.example");
        assert_eq!(with_auth.port, 8080);
        let expected = base64::engine::general_purpose::STANDARD.encode("user:secret");
        assert_eq!(with_auth.auth.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn rejects_unnecessary_or_unsupported_schemes() {
        assert!(parse_upstream("socks5://host:1080").is_err());
        assert!(parse_upstream("https://host:443").is_err());
        assert!(parse_upstream("").is_err());
    }

    #[test]
    fn pool_rotates_round_robin() {
        let pool = ProxyPool::new(vec![parse_ok("a:1"), parse_ok("b:2")]);
        assert_eq!(pool.next_upstream().unwrap().host, "a");
        assert_eq!(pool.next_upstream().unwrap().host, "b");
        assert_eq!(pool.next_upstream().unwrap().host, "a");
    }

    /// A mock upstream HTTP proxy that records the headers it was sent.
    async fn mock_upstream(listener: TcpListener) -> Arc<Mutex<Vec<String>>> {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorder = Arc::clone(&seen);
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let recorder = Arc::clone(&recorder);
                tokio::spawn(async move {
                    let head = read_head(&mut socket).await.unwrap_or_default();
                    recorder.lock().unwrap().push(head.clone());
                    if head.starts_with("CONNECT") {
                        let _ = socket
                            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                            .await;
                        let mut echo = [0u8; 16];
                        if let Ok(read) = socket.read(&mut echo).await {
                            let _ = socket.write_all(&echo[..read]).await;
                        }
                    } else {
                        let _ = socket
                            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi")
                            .await;
                    }
                });
            }
        });
        seen
    }

    #[tokio::test]
    async fn tunnels_connect_to_the_upstream() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let seen = mock_upstream(listener).await;
        let gateway = ProxyGateway::start(vec![parse_ok(&format!("http://user:pw@{addr}"))])
            .await
            .unwrap();

        let mut client = TcpStream::connect(gateway.addr).await.unwrap();
        client
            .write_all(b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\n")
            .await
            .unwrap();
        let head = read_head(&mut client).await.unwrap();
        assert!(head.starts_with("HTTP/1.1 200"), "unexpected: {head}");

        // Tunnel is transparent: bytes written flow through to the mock.
        client.write_all(b"ping").await.unwrap();
        let mut echo = [0u8; 4];
        client.read_exact(&mut echo).await.unwrap();
        assert_eq!(&echo, b"ping");

        let recorded = seen.lock().unwrap().clone();
        assert!(recorded[0].starts_with("CONNECT example.com:443"));
        assert!(
            recorded[0].contains("Proxy-Authorization: Basic "),
            "credentials must be injected: {}",
            recorded[0]
        );
    }

    #[tokio::test]
    async fn forwards_absolute_form_http() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let _seen = mock_upstream(listener).await;
        let gateway = ProxyGateway::start(vec![parse_ok(&format!("http://{addr}"))])
            .await
            .unwrap();

        let mut client = TcpStream::connect(gateway.addr).await.unwrap();
        client
            .write_all(b"GET http://example.com/ HTTP/1.1\r\nHost: example.com\r\n\r\n")
            .await
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).await.unwrap();
        assert!(response.contains("200 OK"), "unexpected: {response}");
        assert!(response.ends_with("hi"));
    }
}
