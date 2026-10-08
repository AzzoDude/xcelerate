//! Media discovery and download.
//!
//! [`Page::media_json`] enumerates what the page references - `<video>` and
//! `<audio>` sources, `<img>` (including `srcset`), and CSS background images -
//! resolved to absolute URLs.
//!
//! [`Page::save_url`] streams any URL to a file through the *browser's* network
//! stack (`Network.loadNetworkResource` + `IO.read`), so cookies and headers
//! apply, CORS never does, and the bytes do not have to fit in memory.
//!
//! [`Page::grab`] is the smart front door: a direct file is streamed as-is, an
//! HLS playlist (`#EXTM3U`) is fetched **segment by segment and concatenated**
//! (so `.m3u8` video "just downloads"), and a DASH (`.mpd`) manifest or an HTML
//! page is reported as unsupported so the CLI can hand it to `yt-dlp`.
//!
//! Honest limits: encrypted HLS (`#EXT-X-KEY`, AES-128) is rejected, and DRM
//! (Widevine/PlayReady) is never touched. Adaptive DASH with per-site signature
//! games (YouTube) is best handled by `yt-dlp`.

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;
use std::path::Path;

/// Walks the rendered DOM and returns `{ count, media: [...] }` as a value.
///
/// The CSS-background scan is capped so a huge page stays responsive.
const MEDIA_JS: &str = r#"(() => {
  const out = [];
  const seen = new Set();
  const streamingSeen = new Set();
  const abs = (u) => { try { return new URL(u, document.baseURI).href; } catch (e) { return null; } };
  const add = (kind, url, extra) => {
    if (!url) return;
    if (url.startsWith('blob:') || url.startsWith('data:')) {
      if (streamingSeen.has(kind)) return;
      streamingSeen.add(kind);
      out.push(Object.assign({ kind: kind, url: null, streaming: true }, extra || {}));
      return;
    }
    if (seen.has(url)) return;
    seen.add(url);
    out.push(Object.assign({ kind: kind, url: url }, extra || {}));
  };
  const splitSrcset = (value) => {
    if (!value) return;
    for (const part of value.split(',')) {
      const u = part.trim().split(/\s+/)[0];
      if (u) add('image', abs(u), {});
    }
  };
  for (const v of document.querySelectorAll('video')) {
    add('video', v.currentSrc || v.src, { poster: v.poster ? abs(v.poster) : null });
    for (const s of v.querySelectorAll('source')) add('video', abs(s.src), { type: s.type || null });
  }
  for (const a of document.querySelectorAll('audio')) {
    add('audio', a.currentSrc || a.src, {});
    for (const s of a.querySelectorAll('source')) add('audio', abs(s.src), { type: s.type || null });
  }
  for (const img of document.querySelectorAll('img')) {
    add('image', abs(img.currentSrc || img.src), { width: img.naturalWidth, height: img.naturalHeight });
    if (img.srcset) splitSrcset(img.srcset);
  }
  for (const s of document.querySelectorAll('source[srcset]')) splitSrcset(s.getAttribute('srcset'));
  const all = document.querySelectorAll('*');
  const cap = Math.min(all.length, 4000);
  for (let i = 0; i < cap; i++) {
    const bg = getComputedStyle(all[i]).backgroundImage;
    if (bg && bg !== 'none') {
      const m = bg.match(/url\((?:["']?)([^"')]+)(?:["']?)\)/);
      if (m) add('image', abs(m[1]), { background: true });
    }
  }
  return { count: out.length, media: out };
})()"#;

/// An opened network resource: its response content type and a CDP IO stream.
struct OpenResource {
    content_type: Option<String>,
    handle: String,
}

impl Page {
    /// Enumerates the media the page references, as JSON.
    ///
    /// Each entry is `{ kind, url, ... }`, where `kind` is `video`, `audio`, or
    /// `image`. A `blob:`/`data:` source (Media Source Extensions) is reported as
    /// `{ "streaming": true, "url": null }` because it has no fetchable URL.
    pub async fn media_json(&self) -> XcelerateResult<String> {
        self.evaluate_json(MEDIA_JS.to_string()).await
    }

    /// Streams `url` to `path` through the browser and returns the bytes written.
    pub async fn save_url(&self, url: String, path: String) -> XcelerateResult<u64> {
        let resource = self.open_resource(&url).await?;
        self.read_resource_to_file(&resource.handle, &path).await
    }

    /// Downloads any media URL: a direct file is streamed to `path`, an HLS
    /// playlist (`#EXTM3U`) is fetched segment by segment, and a DASH manifest
    /// (`.mpd`) is assembled from its segments — all natively, with no external
    /// tool.
    ///
    /// Returns a human-readable summary (a DASH stream with separate audio and
    /// video writes two files). Encrypted HLS and DRM are reported as
    /// [`XcelerateError::Unsupported`].
    pub async fn grab(&self, url: String, path: String) -> XcelerateResult<String> {
        let resource = self.open_resource(&url).await?;
        let content_type = resource.content_type.clone().unwrap_or_default();
        let path_only = url
            .split(['?', '#'])
            .next()
            .unwrap_or(&url)
            .to_ascii_lowercase();
        let is_hls = content_type.contains("mpegurl") || path_only.ends_with(".m3u8");
        let is_dash = content_type.contains("dash+xml") || path_only.ends_with(".mpd");
        let is_html = content_type.contains("text/html");

        if is_dash {
            let manifest = self.read_resource_to_vec(&resource.handle).await?;
            self.close_resource(&resource.handle).await;
            return self.download_dash(&url, &manifest, &path).await;
        }
        if is_html {
            self.close_resource(&resource.handle).await;
            return Err(XcelerateError::Unsupported(
                "this URL is a web page, not a media file; open it and use `media` (or the network \
                 log) to find the page's `.mpd`/`.m3u8` URL, then download that"
                    .to_string(),
            ));
        }
        if is_hls {
            let manifest = self.read_resource_to_vec(&resource.handle).await?;
            self.close_resource(&resource.handle).await;
            let bytes = self.download_hls(&url, &manifest, &path).await?;
            return Ok(format!("saved {path} ({bytes} bytes)"));
        }

        let bytes = self.read_resource_to_file(&resource.handle, &path).await?;
        Ok(format!("saved {path} ({bytes} bytes)"))
    }

    /// Opens a URL with `Network.loadNetworkResource` (credentials included).
    async fn open_resource(&self, url: &str) -> XcelerateResult<OpenResource> {
        use browser_protocol::network;

        let loaded = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                network::LoadNetworkResourceParams {
                    frame_id: Some(self.main_frame_id().await?.into()),
                    url: url.into(),
                    options: network::LoadNetworkResourceOptions {
                        disable_cache: false,
                        include_credentials: true,
                    },
                },
            )
            .await?;

        let resource = loaded.resource;
        if !resource.success {
            return Err(XcelerateError::NotFound(format!(
                "could not fetch {url}: {}",
                resource
                    .net_error_name
                    .as_deref()
                    .unwrap_or("network error")
            )));
        }
        let content_type = resource.headers.as_ref().and_then(|headers| {
            headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case("content-type"))
                .and_then(|(_, value)| value.as_str())
                .map(str::to_ascii_lowercase)
        });
        let handle = resource
            .stream
            .ok_or(XcelerateError::InternalError)?
            .into_owned();
        Ok(OpenResource {
            content_type,
            handle,
        })
    }

    async fn close_resource(&self, handle: &str) {
        let _ = self
            .client
            .execute_with_session(
                Some(&self.session_id),
                browser_protocol::io::CloseParams {
                    handle: handle.into(),
                },
            )
            .await;
    }

    /// Reads a chunk of `handle`, decoding it when it arrives base64-encoded.
    async fn read_chunk(
        &self,
        handle: &str,
    ) -> XcelerateResult<browser_protocol::io::ReadReturns<'_>> {
        Ok(self
            .client
            .execute_with_session(
                Some(&self.session_id),
                browser_protocol::io::ReadParams {
                    handle: handle.into(),
                    offset: None,
                    size: Some(4 * 1024 * 1024),
                },
            )
            .await?)
    }

    fn decode_chunk(&self, chunk: &browser_protocol::io::ReadReturns) -> XcelerateResult<Vec<u8>> {
        if chunk.base64_encoded.unwrap_or(false) {
            self.decode_base64(chunk.data.to_string())
        } else {
            Ok(chunk.data.as_bytes().to_vec())
        }
    }

    /// Streams a whole resource to a file.
    async fn read_resource_to_file(&self, handle: &str, path: &str) -> XcelerateResult<u64> {
        create_parent(path).await?;
        let mut file = tokio::fs::File::create(path).await.map_err(io_error)?;
        use tokio::io::AsyncWriteExt;
        let mut total: u64 = 0;
        loop {
            let chunk = self.read_chunk(handle).await?;
            let bytes = self.decode_chunk(&chunk)?;
            if !bytes.is_empty() {
                file.write_all(&bytes).await.map_err(io_error)?;
                total += bytes.len() as u64;
            }
            if chunk.eof {
                break;
            }
        }
        self.close_resource(handle).await;
        Ok(total)
    }

    /// Reads a whole resource into memory (for manifests).
    async fn read_resource_to_vec(&self, handle: &str) -> XcelerateResult<Vec<u8>> {
        let mut out = Vec::new();
        loop {
            let chunk = self.read_chunk(handle).await?;
            out.extend_from_slice(&self.decode_chunk(&chunk)?);
            if chunk.eof {
                break;
            }
        }
        Ok(out)
    }

    /// Fetches a whole URL into memory through the browser.
    pub(crate) async fn fetch_bytes(&self, url: &str) -> XcelerateResult<Vec<u8>> {
        let resource = self.open_resource(url).await?;
        let bytes = self.read_resource_to_vec(&resource.handle).await?;
        self.close_resource(&resource.handle).await;
        Ok(bytes)
    }

    /// Downloads an HLS playlist: follows the master to its best variant, then
    /// concatenates the init segment and every media segment in order.
    async fn download_hls(
        &self,
        base_url: &str,
        manifest: &[u8],
        out_path: &str,
    ) -> XcelerateResult<u64> {
        let mut playlist = String::from_utf8_lossy(manifest).to_string();
        let mut base = base_url.to_string();

        // A master playlist lists variant streams; pick the highest bandwidth.
        if playlist.contains("#EXT-X-STREAM-INF") {
            let variant = pick_best_variant(&playlist, base_url).ok_or_else(|| {
                XcelerateError::NotFound("HLS master playlist had no playable variant".to_string())
            })?;
            let bytes = self.fetch_bytes(&variant).await?;
            playlist = String::from_utf8_lossy(&bytes).to_string();
            base = variant;
        }

        if has_encryption(&playlist) {
            return Err(XcelerateError::Unsupported(
                "this HLS stream is encrypted (EXT-X-KEY, AES-128); encrypted streams are not \
                 supported"
                    .to_string(),
            ));
        }

        let mut urls: Vec<String> = Vec::new();
        if let Some(init) = map_uri(&playlist) {
            urls.push(resolve_url(&base, &init));
        }
        urls.extend(
            segment_uris(&playlist)
                .iter()
                .map(|u| resolve_url(&base, u)),
        );
        if urls.is_empty() {
            return Err(XcelerateError::NotFound(
                "no segments found in the HLS playlist".to_string(),
            ));
        }

        // Concatenate the init segment and every media segment to the target.
        create_parent(out_path).await?;
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(out_path).await.map_err(io_error)?;
        let mut total: u64 = 0;
        for url in &urls {
            let bytes = self.fetch_bytes(url).await?;
            file.write_all(&bytes).await.map_err(io_error)?;
            total += bytes.len() as u64;
        }
        Ok(total)
    }

    /// Downloads a DASH (`.mpd`) stream natively: picks the best video and audio
    /// representations, then concatenates each track's init + media segments.
    ///
    /// A muxed representation (or a video-only stream) writes one file. A stream
    /// with separate audio needs two containers; the audio goes to
    /// `<name>.audio.m4a` beside `path`.
    async fn download_dash(
        &self,
        base_url: &str,
        manifest: &[u8],
        out_path: &str,
    ) -> XcelerateResult<String> {
        let text = String::from_utf8_lossy(manifest);
        let (video, audio) =
            super::dash::plan(&text, base_url).map_err(XcelerateError::Unsupported)?;
        if video.is_none() && audio.is_none() {
            return Err(XcelerateError::NotFound(
                "the DASH manifest has no downloadable representation".to_string(),
            ));
        }

        let mut reports = Vec::new();
        if let Some(track) = &video {
            let bytes = self.write_track(track, out_path).await?;
            reports.push(format!("{out_path} ({bytes} bytes)"));
        }
        if let Some(track) = &audio {
            let audio_path = if video.is_some() {
                split_audio_path(out_path)
            } else {
                out_path.to_string()
            };
            let bytes = self.write_track(track, &audio_path).await?;
            reports.push(format!("{audio_path} ({bytes} bytes)"));
        }
        Ok(format!("saved {}", reports.join(" + ")))
    }

    /// Concatenates one DASH track's init segment and media segments into `path`.
    async fn write_track(&self, track: &super::dash::Track, path: &str) -> XcelerateResult<u64> {
        create_parent(path).await?;
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(path).await.map_err(io_error)?;
        let mut total: u64 = 0;
        if let Some(init) = &track.init {
            let bytes = self.fetch_bytes(init).await?;
            file.write_all(&bytes).await.map_err(io_error)?;
            total += bytes.len() as u64;
        }
        for segment in &track.segments {
            let bytes = self.fetch_bytes(segment).await?;
            file.write_all(&bytes).await.map_err(io_error)?;
            total += bytes.len() as u64;
        }
        Ok(total)
    }

    /// The id of the page's main frame, from `Page.getFrameTree`.
    async fn main_frame_id(&self) -> XcelerateResult<String> {
        let res = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Page.getFrameTree",
                serde_json::json!({}),
            )
            .await?;
        res.pointer("/frameTree/frame/id")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .ok_or(XcelerateError::InternalError)
    }
}

pub(crate) async fn create_parent(path: &str) -> XcelerateResult<()> {
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent).await.map_err(io_error)?;
    }
    Ok(())
}

pub(crate) fn io_error(error: std::io::Error) -> XcelerateError {
    XcelerateError::NotFound(format!("io error: {error}"))
}

/// Whether an HLS playlist has any `#EXT-X-KEY` other than `METHOD=NONE`.
fn has_encryption(playlist: &str) -> bool {
    playlist.lines().any(|line| {
        let line = line.trim();
        line.starts_with("#EXT-X-KEY") && !line.to_ascii_uppercase().contains("METHOD=NONE")
    })
}

/// The `URI="..."` of an `#EXT-X-MAP` init segment, if present.
fn map_uri(playlist: &str) -> Option<String> {
    playlist.lines().find_map(|line| {
        let line = line.trim();
        (line.starts_with("#EXT-X-MAP"))
            .then(|| quoted_attr(line, "URI"))
            .flatten()
    })
}

/// The media segment URIs, in order: non-comment, non-blank lines.
fn segment_uris(playlist: &str) -> Vec<String> {
    playlist
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// Picks the highest-`BANDWIDTH` variant from an HLS master playlist.
fn pick_best_variant(playlist: &str, base_url: &str) -> Option<String> {
    let mut best: Option<(u64, String)> = None;
    let mut pending_bandwidth: Option<u64> = None;
    for line in playlist.lines() {
        let line = line.trim();
        if let Some(attrs) = line.strip_prefix("#EXT-X-STREAM-INF:") {
            pending_bandwidth = bandwidth(attrs);
        } else if !line.is_empty() && !line.starts_with('#') {
            let bw = pending_bandwidth.take().unwrap_or(0);
            if best.as_ref().is_none_or(|(current, _)| bw >= *current) {
                best = Some((bw, resolve_url(base_url, line)));
            }
        }
    }
    best.map(|(_, url)| url)
}

/// The `BANDWIDTH=NNN` attribute of a `#EXT-X-STREAM-INF` tag.
fn bandwidth(attrs: &str) -> Option<u64> {
    for part in attrs.split(',') {
        if let Some((key, value)) = part.split_once('=')
            && key.trim().eq_ignore_ascii_case("BANDWIDTH")
        {
            return value.trim().parse().ok();
        }
    }
    None
}

/// Extracts `NAME="value"` from a playlist tag.
fn quoted_attr(line: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = line.find(&needle)? + needle.len();
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

/// Resolves a possibly-relative playlist URI against `base`.
fn resolve_url(base: &str, uri: &str) -> String {
    let uri = uri.trim();
    if uri.starts_with("http://") || uri.starts_with("https://") {
        return uri.to_string();
    }
    if let Some(rest) = uri.strip_prefix("//") {
        let scheme = base.split(':').next().unwrap_or("https");
        return format!("{scheme}://{rest}");
    }
    let base = base.split(['?', '#']).next().unwrap_or(base);
    if uri.starts_with('/')
        && let Some(scheme_end) = base.find("://")
    {
        let after = &base[scheme_end + 3..];
        let host_end = after
            .find('/')
            .map(|i| scheme_end + 3 + i)
            .unwrap_or(base.len());
        return format!("{}{}", &base[..host_end], uri);
    }
    match base.rfind('/') {
        Some(index) => format!("{}/{}", &base[..index], uri),
        None => format!("{base}/{uri}"),
    }
}

/// The sibling path a separate DASH audio track is written to, e.g.
/// `movie.mp4` -> `movie.audio.m4a`.
pub(crate) fn split_audio_path(out: &str) -> String {
    let last_sep = out.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    match out.rfind('.') {
        Some(index) if index > last_sep => format!("{}.audio.m4a", &out[..index]),
        _ => format!("{out}.audio.m4a"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_playlist_uris() {
        assert_eq!(
            resolve_url("https://cdn.example/hls/master.m3u8", "seg1.ts"),
            "https://cdn.example/hls/seg1.ts"
        );
        assert_eq!(
            resolve_url(
                "https://cdn.example/hls/master.m3u8?token=1",
                "/root/seg.ts"
            ),
            "https://cdn.example/root/seg.ts"
        );
        assert_eq!(
            resolve_url("https://cdn.example/a/b.m3u8", "https://other/x.ts"),
            "https://other/x.ts"
        );
        assert_eq!(
            resolve_url("https://cdn.example/a/b.m3u8", "//other/x.ts"),
            "https://other/x.ts"
        );
    }

    #[test]
    fn picks_the_highest_bandwidth_variant() {
        let master = "#EXTM3U\n\
            #EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=640x360\n\
            low/index.m3u8\n\
            #EXT-X-STREAM-INF:BANDWIDTH=2500000,RESOLUTION=1280x720\n\
            high/index.m3u8\n";
        assert_eq!(
            pick_best_variant(master, "https://cdn/hls/master.m3u8").as_deref(),
            Some("https://cdn/hls/high/index.m3u8")
        );
    }

    #[test]
    fn lists_segments_and_finds_map() {
        let media = "#EXTM3U\n\
            #EXT-X-MAP:URI=\"init.mp4\"\n\
            #EXTINF:4.0,\n\
            seg0.m4s\n\
            #EXTINF:4.0,\n\
            seg1.m4s\n";
        assert_eq!(map_uri(media).as_deref(), Some("init.mp4"));
        assert_eq!(segment_uris(media), vec!["seg0.m4s", "seg1.m4s"]);
        assert!(!has_encryption(media));
    }

    #[test]
    fn flags_encryption_but_allows_method_none() {
        assert!(has_encryption(
            "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\"\n"
        ));
        assert!(!has_encryption("#EXTM3U\n#EXT-X-KEY:METHOD=NONE\n"));
    }
}
