//! In-process HTTPS client, compiled only with the `http` feature.
//!
//! This is deliberately *opt-in and CLI-only*. The core `xcelerate` crate never
//! links it, so the cdylib and every language binding keep their minimal
//! dependency surface. Enable with `cargo build -p xcelerate-cli --features http`.
//!
//! Use it for things the browser should not do in-band: fetching an API's JSON,
//! or streaming a binary/asset download straight to disk.

use std::path::Path;

/// A shared client: connection pooling and TLS are set up once.
fn client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent(concat!("xcelerate/", env!("CARGO_PKG_VERSION")))
        .build()
}

/// The client used by `request` / browserless HTTP: it never follows redirects.
///
/// This is an SSRF control: the caller validates the target host once, before
/// the request, so an auto-followed `302` to `http://127.0.0.1/` would otherwise
/// reach loopback behind the guard's back. A script that wants a redirect chain
/// must issue each hop explicitly.
fn request_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .user_agent(concat!("xcelerate/", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::none())
        .build()
}

/// Fetches `url` and returns the body. JSON is pretty-printed so an API call is
/// readable; anything else is returned verbatim.
pub async fn fetch(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = client()?.get(url).send().await?;
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        let snippet: String = body.chars().take(200).collect();
        return Err(format!("HTTP {status} for {url}: {snippet}").into());
    }

    Ok(match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(value) => serde_json::to_string_pretty(&value)?,
        Err(_) => body,
    })
}

/// Streams `url` to `path`, returning the number of bytes written.
pub async fn download(url: &str, path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;

    let response = client()?.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {status} for {url}").into());
    }

    let total = response.content_length();
    let mut file = tokio::fs::File::create(path).await?;
    let mut stream = response.bytes_stream();
    let mut written: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        written += chunk.len() as u64;
        if let Some(total) = total {
            print!("\r{written} / {total} bytes");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }
    if total.is_some() {
        println!();
    }
    file.flush().await?;
    Ok(written)
}

/// A general HTTP request for the XCL `request` command and browserless mode.
///
/// Returns `(status_code, response_body_text)`. `headers` is a list of
/// `(name, value)` pairs. This is request-only automation: no browser, no DOM.
///
/// SSRF is the caller's responsibility: the XCL engine checks the domain policy
/// and private-range rules *before* invoking this.
pub async fn request(
    method: &str,
    url: &str,
    headers: Vec<(String, String)>,
    body: String,
) -> Result<(u16, String), Box<dyn std::error::Error>> {
    let client = request_client()?;
    let method = reqwest::Method::from_bytes(method.as_bytes())?;
    let mut builder = client.request(method, url);
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    if !body.is_empty() {
        builder = builder.body(body);
    }
    let response = builder.send().await?;
    let status = response.status().as_u16();
    let text = response.text().await?;
    Ok((status, text))
}
