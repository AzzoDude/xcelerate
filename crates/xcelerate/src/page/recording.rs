//! Session video recording.
//!
//! Recording is driven by the CDP screencast: [`Page::start_video`] subscribes
//! to `Page.screencastFrame`, acknowledges every frame (Chrome stops streaming
//! otherwise), and writes the JPEG payloads to a scratch directory.
//!
//! [`Page::stop_video`] then muxes those frames into a video. Two back ends are
//! supported:
//!
//! * **Native (always available)** - a JPEG frame sequence is wrapped in a
//!   Motion-JPEG AVI by [`avi::write_mjpeg_avi`]. No external tooling is needed.
//! * **ffmpeg (optional)** - if `ffmpeg` is on `PATH` and the requested
//!   container is `.mp4`/`.mov`/`.mkv`/`.webm`, the frames are muxed with a real
//!   codec (H.264/VP9) at their true capture timestamps. When ffmpeg is missing
//!   we fall back to the native AVI writer rather than failing.
//!
//! These methods live in a plain `impl Page` block (no `#[uniffi::export]`), so
//! they are available to Rust, the CLI, and the MCP server without changing the
//! generated bindings' UniFFI checksums.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tokio::sync::Notify;

use crate::error::{XcelerateError, XcelerateResult};

use super::Page;

/// Options for a video recording.
#[derive(Clone, Debug)]
pub struct VideoOptions {
    /// JPEG quality for each captured frame (0-100). Ignored by PNG.
    pub quality: u32,
    /// Downscale frames to at most this width (0 keeps the native width).
    pub max_width: u32,
    /// Downscale frames to at most this height (0 keeps the native height).
    pub max_height: u32,
    /// Output frame rate for the native AVI back end.
    pub fps: u32,
    /// Use ffmpeg for `.mp4`/`.mov`/`.mkv`/`.webm` outputs when it is available.
    pub ffmpeg: bool,
}

impl Default for VideoOptions {
    fn default() -> Self {
        Self {
            quality: 80,
            max_width: 0,
            max_height: 0,
            fps: 12,
            ffmpeg: true,
        }
    }
}

/// A captured frame's position on the recording timeline.
#[derive(Clone, Copy)]
struct FrameStamp {
    index: u32,
    at_ms: u64,
}

/// Live recording state stored on the page between `start_video` and `stop_video`.
pub(crate) struct VideoRecording {
    output: PathBuf,
    frames_dir: PathBuf,
    options: VideoOptions,
    stop: Arc<AtomicBool>,
    notify: Arc<Notify>,
    frames: Arc<std::sync::Mutex<Vec<FrameStamp>>>,
    dims: Arc<std::sync::Mutex<Option<(u32, u32)>>>,
    task: tokio::task::JoinHandle<()>,
}

static RECORDING_SEQ: AtomicU64 = AtomicU64::new(0);

fn scratch_dir() -> PathBuf {
    let seq = RECORDING_SEQ.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "xcelerate-video-{}-{seq}-{nanos}",
        std::process::id()
    ))
}

impl Page {
    /// Starts recording the page to `output`.
    ///
    /// Returns once the screencast is confirmed to be streaming. Call
    /// [`Page::stop_video`] to finalize the file; the returned path may differ
    /// from `output` if an ffmpeg-only container was requested but ffmpeg is
    /// unavailable (the native writer then produces a sibling `.avi`).
    pub async fn start_video(&self, output: String) -> XcelerateResult<()> {
        self.start_video_with_options(output, VideoOptions::default())
            .await
    }

    /// Like [`Page::start_video`], with explicit [`VideoOptions`].
    pub async fn start_video_with_options(
        &self,
        output: String,
        options: VideoOptions,
    ) -> XcelerateResult<()> {
        let mut guard = self.recording.lock().await;
        if guard.is_some() {
            return Err(XcelerateError::Unsupported(
                "a video recording is already in progress".to_string(),
            ));
        }

        let frames_dir = scratch_dir();
        if let Err(e) = tokio::fs::create_dir_all(&frames_dir).await {
            return Err(XcelerateError::Unsupported(format!(
                "could not create recording scratch directory: {e}"
            )));
        }

        let stop = Arc::new(AtomicBool::new(false));
        let notify = Arc::new(Notify::new());
        let frames = Arc::new(std::sync::Mutex::new(Vec::new()));
        let dims = Arc::new(std::sync::Mutex::new(None));
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

        let task = tokio::spawn(run_recording(
            Arc::clone(&self.client),
            self.session_id.clone(),
            frames_dir.clone(),
            options.clone(),
            Arc::clone(&frames),
            Arc::clone(&dims),
            Arc::clone(&notify),
            Arc::clone(&stop),
            ready_tx,
        ));

        match ready_rx.await {
            Ok(Ok(())) => {}
            Ok(Err(message)) => {
                let _ = task.await;
                let _ = tokio::fs::remove_dir_all(&frames_dir).await;
                return Err(XcelerateError::Unsupported(message));
            }
            Err(_) => {
                let _ = task.await;
                let _ = tokio::fs::remove_dir_all(&frames_dir).await;
                return Err(XcelerateError::InternalError);
            }
        }

        *guard = Some(VideoRecording {
            output: PathBuf::from(output),
            frames_dir,
            options,
            stop,
            notify,
            frames,
            dims,
            task,
        });
        Ok(())
    }

    /// Stops the current recording and muxes the captured frames.
    ///
    /// Returns the path of the written video, or `None` if nothing was being
    /// recorded. The final path may differ from the one passed to
    /// [`Page::start_video`] when the native back end is used (see there).
    pub async fn stop_video(&self) -> XcelerateResult<Option<String>> {
        let recording = {
            let mut guard = self.recording.lock().await;
            guard.take()
        };
        let Some(recording) = recording else {
            return Ok(None);
        };

        recording.stop.store(true, Ordering::SeqCst);
        recording.notify.notify_one();
        // Take the handle so the other fields stay usable for the mux step.
        let task = recording.task;
        let _ = task.await;

        let stamps: Vec<FrameStamp> = recording
            .frames
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default();

        if stamps.is_empty() {
            let _ = tokio::fs::remove_dir_all(&recording.frames_dir).await;
            return Err(XcelerateError::NotFound(
                "no frames were captured during recording".to_string(),
            ));
        }

        let result = finalize(
            &recording.frames_dir,
            &recording.output,
            &recording.options,
            &recording.dims,
            &stamps,
        )
        .await;
        let _ = tokio::fs::remove_dir_all(&recording.frames_dir).await;
        result.map(|path| Some(path.to_string_lossy().into_owned()))
    }

    /// Whether a recording is currently in progress.
    pub async fn is_recording(&self) -> bool {
        self.recording.lock().await.is_some()
    }
}

fn frame_name(index: u32) -> String {
    format!("frame_{index:06}.jpg")
}

/// Background task: stream screencast frames to disk until signalled to stop.
#[allow(clippy::too_many_arguments)]
async fn run_recording(
    client: Arc<crate::CdpClient>,
    session_id: String,
    frames_dir: PathBuf,
    options: VideoOptions,
    frames: Arc<std::sync::Mutex<Vec<FrameStamp>>>,
    dims: Arc<std::sync::Mutex<Option<(u32, u32)>>>,
    notify: Arc<Notify>,
    stop: Arc<AtomicBool>,
    ready: tokio::sync::oneshot::Sender<Result<(), String>>,
) {
    // The Page domain must be enabled for screencast events to flow.
    let _ = client
        .execute_raw_with_session(Some(&session_id), "Page.enable", serde_json::json!({}))
        .await;

    let mut params = serde_json::json!({
        "format": "jpeg",
        "everyNthFrame": 1,
        "quality": options.quality.clamp(1, 100),
    });
    if options.max_width > 0 {
        params["maxWidth"] = serde_json::json!(options.max_width);
    }
    if options.max_height > 0 {
        params["maxHeight"] = serde_json::json!(options.max_height);
    }

    if let Err(e) = client
        .execute_raw_with_session(Some(&session_id), "Page.startScreencast", params)
        .await
    {
        let _ = ready.send(Err(format!("could not start screencast: {e}")));
        stop.store(true, Ordering::SeqCst);
        return;
    }
    let _ = ready.send(Ok(()));

    let started = std::time::Instant::now();
    let mut receiver = client.subscribe();
    let mut index: u32 = 0;

    loop {
        tokio::select! {
            _ = notify.notified() => break,
            event = receiver.recv() => {
                let value = match event {
                    Ok(value) => value,
                    // A slow consumer only misses frames; keep pumping.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                };
                if value.get("sessionId").and_then(|s| s.as_str()) != Some(session_id.as_str()) {
                    continue;
                }
                if value.get("method").and_then(|m| m.as_str()) != Some("Page.screencastFrame") {
                    continue;
                }
                let params = value.get("params").cloned().unwrap_or_default();

                // Ack first: Chrome throttles or stops the stream until the
                // frame is acknowledged, even if writing it fails.
                if let Some(ack) = params.get("sessionId").and_then(|v| v.as_i64()) {
                    let _ = client
                        .execute_raw_with_session(
                            Some(&session_id),
                            "Page.screencastFrameAck",
                            serde_json::json!({ "sessionId": ack }),
                        )
                        .await;
                }

                if let Some((w, h)) = read_dims(&params)
                    && let Ok(mut guard) = dims.lock()
                {
                    *guard = Some((w, h));
                }

                let Some(data) = params.get("data").and_then(|d| d.as_str()) else {
                    continue;
                };
                let Ok(bytes) = decode_base64(data) else {
                    continue;
                };
                let file = frames_dir.join(frame_name(index));
                if tokio::fs::write(&file, &bytes).await.is_err() {
                    continue;
                }
                let at_ms = started.elapsed().as_millis() as u64;
                if let Ok(mut guard) = frames.lock() {
                    guard.push(FrameStamp { index, at_ms });
                }
                index += 1;
            }
        }
    }

    let _ = client
        .execute_raw_with_session(
            Some(&session_id),
            "Page.stopScreencast",
            serde_json::json!({}),
        )
        .await;
}

fn read_dims(params: &serde_json::Value) -> Option<(u32, u32)> {
    let metadata = params.get("metadata")?;
    let width = metadata.get("deviceWidth").and_then(|v| v.as_u64())? as u32;
    let height = metadata.get("deviceHeight").and_then(|v| v.as_u64())? as u32;
    if width == 0 || height == 0 {
        None
    } else {
        Some((width, height))
    }
}

fn decode_base64(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.decode(data)
}

fn is_ffmpeg_container(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("mp4" | "mov" | "mkv" | "webm")
    )
}

/// Mux captured frames into the target file, choosing the back end by container
/// and by whether ffmpeg is available.
async fn finalize(
    frames_dir: &Path,
    output: &Path,
    options: &VideoOptions,
    dims: &std::sync::Mutex<Option<(u32, u32)>>,
    stamps: &[FrameStamp],
) -> XcelerateResult<PathBuf> {
    let mut images = Vec::with_capacity(stamps.len());
    for stamp in stamps {
        let path = frames_dir.join(frame_name(stamp.index));
        match tokio::fs::read(&path).await {
            Ok(bytes) => images.push(bytes),
            Err(_) => continue,
        }
    }
    if images.is_empty() {
        return Err(XcelerateError::NotFound(
            "no readable frames were captured during recording".to_string(),
        ));
    }

    let (width, height) = dims
        .lock()
        .ok()
        .and_then(|guard| *guard)
        .or_else(|| jpeg_dimensions(&images[0]))
        .unwrap_or((1280, 720));

    if options.ffmpeg && is_ffmpeg_container(output) {
        match ffmpeg_encode(frames_dir, stamps, output).await {
            Ok(()) => return Ok(output.to_path_buf()),
            Err(EncodeError::Missing) => {
                // Fall through to the native writer; the caller learns the real
                // path from the return value.
            }
            Err(EncodeError::Failed(message)) => {
                return Err(XcelerateError::Unsupported(format!(
                    "ffmpeg failed to encode the recording: {message}"
                )));
            }
        }
    }

    let native = native_output_path(output);
    let timeline = resample(stamps, options.fps);
    let selected: Vec<&[u8]> = timeline.iter().map(|&i| images[i].as_slice()).collect();
    avi::write_mjpeg_avi(&native, &selected, width, height, options.fps)
        .map_err(|e| XcelerateError::Unsupported(format!("could not write video: {e}")))?;
    Ok(native)
}

fn native_output_path(output: &Path) -> PathBuf {
    if output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("avi"))
        .unwrap_or(false)
    {
        return output.to_path_buf();
    }
    output.with_extension("avi")
}

/// Map captured frames onto a fixed-rate timeline, repeating the most recent
/// frame across gaps so playback matches wall-clock time.
fn resample(stamps: &[FrameStamp], fps: u32) -> Vec<usize> {
    let fps = fps.max(1) as u64;
    let start = stamps[0].at_ms;
    let end = stamps[stamps.len() - 1].at_ms + 1000 / fps;
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let mut step = 0u64;
    loop {
        let t = start + step * 1000 / fps;
        if t >= end {
            break;
        }
        while cursor + 1 < stamps.len() && stamps[cursor + 1].at_ms <= t {
            cursor += 1;
        }
        out.push(cursor);
        step += 1;
    }
    if out.is_empty() {
        out.push(0);
    }
    out
}

enum EncodeError {
    /// ffmpeg is not installed (or not on `PATH`).
    Missing,
    /// ffmpeg ran but exited non-zero.
    Failed(String),
}

async fn ffmpeg_encode(
    frames_dir: &Path,
    stamps: &[FrameStamp],
    output: &Path,
) -> Result<(), EncodeError> {
    let mut list = String::new();
    for (i, stamp) in stamps.iter().enumerate() {
        let next_ms = if i + 1 < stamps.len() {
            stamps[i + 1].at_ms
        } else {
            stamp.at_ms + 40
        };
        let duration_ms = next_ms.saturating_sub(stamp.at_ms).max(1);
        let file = concat_path(&frames_dir.join(frame_name(stamp.index)));
        list.push_str(&format!(
            "file {file}\nduration {:.3}\n",
            duration_ms as f64 / 1000.0
        ));
    }
    // The concat demuxer drops the final frame unless it is listed again.
    if let Some(last) = stamps.last() {
        let file = concat_path(&frames_dir.join(frame_name(last.index)));
        list.push_str(&format!("file {file}\n"));
    }
    let list_path = frames_dir.join("frames.txt");
    tokio::fs::write(&list_path, list)
        .await
        .map_err(|e| EncodeError::Failed(e.to_string()))?;

    let codec = if output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("webm"))
        .unwrap_or(false)
    {
        "libvpx-vp9"
    } else {
        "libx264"
    };

    let result = tokio::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path)
        .arg("-vf")
        .arg("scale=trunc(iw/2)*2:trunc(ih/2)*2")
        .arg("-c:v")
        .arg(codec)
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-vsync")
        .arg("vfr")
        .arg(output)
        .output()
        .await;

    match result {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(EncodeError::Missing),
        Err(e) => Err(EncodeError::Failed(e.to_string())),
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(EncodeError::Failed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        )),
    }
}

fn concat_path(path: &Path) -> String {
    // The concat demuxer wants forward slashes and single-quoted paths.
    let normalized = path.to_string_lossy().replace('\\', "/");
    format!("'{}'", normalized.replace('\'', "'\\''"))
}

/// Reads the pixel dimensions from a JPEG's SOF marker. Used as a fallback when
/// the screencast metadata is unavailable.
fn jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 9 <= data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        if marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            i += 2;
            continue;
        }
        if marker == 0xD9 || marker == 0xDA {
            break;
        }
        let length = ((data[i + 2] as usize) << 8) | data[i + 3] as usize;
        if length < 2 {
            return None;
        }
        let is_sof =
            (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC;
        if is_sof {
            let height = ((data[i + 5] as u32) << 8) | data[i + 6] as u32;
            let width = ((data[i + 7] as u32) << 8) | data[i + 8] as u32;
            return Some((width, height));
        }
        i += 2 + length;
    }
    None
}

/// A minimal Motion-JPEG AVI muxer.
///
/// It writes just enough of the AVI/RIFF structure (`hdrl`/`avih`/`strl`,
/// a `movi` list of `00dc` JPEG chunks, and an `idx1` index) to produce a file
/// that mainstream players and ffmpeg accept.
mod avi {
    use std::io;
    use std::path::Path;

    fn push_u32(buf: &mut Vec<u8>, value: u32) {
        buf.extend_from_slice(&value.to_le_bytes());
    }

    fn push_i32(buf: &mut Vec<u8>, value: i32) {
        buf.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u16(buf: &mut Vec<u8>, value: u16) {
        buf.extend_from_slice(&value.to_le_bytes());
    }

    /// A RIFF chunk: fourcc, little-endian length, payload, pad byte if odd.
    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + data.len() + 1);
        out.extend_from_slice(tag);
        push_u32(&mut out, data.len() as u32);
        out.extend_from_slice(data);
        if data.len() % 2 == 1 {
            out.push(0);
        }
        out
    }

    /// Writes a Motion-JPEG AVI from JPEG frames.
    pub(super) fn write_mjpeg_avi(
        path: &Path,
        frames: &[&[u8]],
        width: u32,
        height: u32,
        fps: u32,
    ) -> io::Result<()> {
        let fps = fps.max(1);
        let micros_per_frame = (1_000_000f64 / f64::from(fps)).round() as u32;
        let max_frame = frames.iter().map(|f| f.len()).max().unwrap_or(0) as u32;
        let total = frames.len() as u32;

        // avih (AVIMAINHEADER)
        let mut avih = Vec::new();
        push_u32(&mut avih, micros_per_frame);
        push_u32(&mut avih, max_frame.saturating_mul(fps));
        push_u32(&mut avih, 0);
        push_u32(&mut avih, 0x10); // AVIF_HASINDEX
        push_u32(&mut avih, total);
        push_u32(&mut avih, 0);
        push_u32(&mut avih, 1); // one stream
        push_u32(&mut avih, max_frame);
        push_u32(&mut avih, width);
        push_u32(&mut avih, height);
        for _ in 0..4 {
            push_u32(&mut avih, 0);
        }

        // strh (AVISTREAMHEADER, video)
        let mut strh = Vec::new();
        strh.extend_from_slice(b"vids");
        strh.extend_from_slice(b"MJPG");
        push_u32(&mut strh, 0);
        push_u16(&mut strh, 0);
        push_u16(&mut strh, 0);
        push_u32(&mut strh, 0);
        push_u32(&mut strh, 1); // scale
        push_u32(&mut strh, fps); // rate
        push_u32(&mut strh, 0);
        push_u32(&mut strh, total);
        push_u32(&mut strh, max_frame);
        push_u32(&mut strh, 0xFFFF_FFFF); // quality: -1
        push_u32(&mut strh, 0);
        push_u16(&mut strh, 0);
        push_u16(&mut strh, 0);
        push_u16(&mut strh, width.min(u32::from(u16::MAX)) as u16);
        push_u16(&mut strh, height.min(u32::from(u16::MAX)) as u16);

        // strf (BITMAPINFOHEADER)
        let mut strf = Vec::new();
        push_u32(&mut strf, 40);
        push_i32(&mut strf, width as i32);
        push_i32(&mut strf, height as i32);
        push_u16(&mut strf, 1);
        push_u16(&mut strf, 24);
        strf.extend_from_slice(b"MJPG");
        push_u32(&mut strf, width.saturating_mul(height).saturating_mul(3));
        push_i32(&mut strf, 0);
        push_i32(&mut strf, 0);
        push_u32(&mut strf, 0);
        push_u32(&mut strf, 0);

        let mut strl_body = chunk(b"strh", &strh);
        strl_body.extend_from_slice(&chunk(b"strf", &strf));
        let mut strl = Vec::new();
        strl.extend_from_slice(b"LIST");
        push_u32(&mut strl, (4 + strl_body.len()) as u32);
        strl.extend_from_slice(b"strl");
        strl.extend_from_slice(&strl_body);

        let mut hdrl_body = chunk(b"avih", &avih);
        hdrl_body.extend_from_slice(&strl);
        let mut hdrl = Vec::new();
        hdrl.extend_from_slice(b"LIST");
        push_u32(&mut hdrl, (4 + hdrl_body.len()) as u32);
        hdrl.extend_from_slice(b"hdrl");
        hdrl.extend_from_slice(&hdrl_body);

        // movi list: `00dc` chunks, plus the matching idx1 entries.
        let mut movi_body = Vec::new();
        let mut index = Vec::new();
        for frame in frames {
            // idx1 offsets are relative to the `movi` fourcc.
            let offset = 4 + movi_body.len() as u32;
            movi_body.extend_from_slice(b"00dc");
            push_u32(&mut movi_body, frame.len() as u32);
            movi_body.extend_from_slice(frame);
            if frame.len() % 2 == 1 {
                movi_body.push(0);
            }
            index.extend_from_slice(b"00dc");
            push_u32(&mut index, 0x10); // AVIIF_KEYFRAME
            push_u32(&mut index, offset);
            push_u32(&mut index, frame.len() as u32);
        }
        let mut movi = Vec::new();
        movi.extend_from_slice(b"LIST");
        push_u32(&mut movi, (4 + movi_body.len()) as u32);
        movi.extend_from_slice(b"movi");
        movi.extend_from_slice(&movi_body);

        let idx1 = chunk(b"idx1", &index);

        let mut riff = Vec::new();
        riff.extend_from_slice(b"RIFF");
        push_u32(&mut riff, 0); // patched below
        riff.extend_from_slice(b"AVI ");
        riff.extend_from_slice(&hdrl);
        riff.extend_from_slice(&movi);
        riff.extend_from_slice(&idx1);
        let riff_size = (riff.len() - 8) as u32;
        riff[4..8].copy_from_slice(&riff_size.to_le_bytes());

        std::fs::write(path, &riff)
    }

    #[cfg(test)]
    mod tests {
        use super::write_mjpeg_avi;

        fn read_u32(bytes: &[u8], at: usize) -> u32 {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        }

        #[test]
        fn writes_a_well_formed_mjpeg_avi() {
            let dir =
                std::env::temp_dir().join(format!("xcelerate-avi-test-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("out.avi");
            let frame_a = vec![0xFF, 0xD8, 0x01, 0x02, 0xD9];
            let frame_b = vec![0xFF, 0xD8, 0x03, 0x04, 0x05, 0xD9];
            write_mjpeg_avi(&path, &[&frame_a, &frame_b], 64, 48, 10).unwrap();

            let bytes = std::fs::read(&path).unwrap();
            assert_eq!(&bytes[0..4], b"RIFF");
            assert_eq!(&bytes[8..12], b"AVI ");
            assert_eq!(read_u32(&bytes, 4) as usize, bytes.len() - 8);
            // Two `00dc` frame headers and two idx1 entries.
            let movi = bytes.windows(4).filter(|w| *w == b"movi").count();
            assert_eq!(movi, 1);
            let dc = bytes.windows(4).filter(|w| *w == b"00dc").count();
            assert_eq!(dc, 4); // 2 in movi + 2 in idx1
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameStamp, jpeg_dimensions, resample};

    #[test]
    fn resample_repeats_frames_across_gaps() {
        let stamps = vec![
            FrameStamp { index: 0, at_ms: 0 },
            FrameStamp {
                index: 1,
                at_ms: 500,
            },
        ];
        // 10 fps over ~0.6s -> six frames, the first five from frame 0.
        let timeline = resample(&stamps, 10);
        assert_eq!(timeline, vec![0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn reads_jpeg_dimensions_from_sof() {
        // SOI, then SOF0 with length 0x11, precision 8, height 48, width 64.
        let jpeg = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08, 0x00, 0x30, 0x00, 0x40, 0xFF, 0xD9,
        ];
        assert_eq!(jpeg_dimensions(&jpeg), Some((64, 48)));
    }

    #[test]
    fn rejects_non_jpeg() {
        assert_eq!(jpeg_dimensions(b"not a jpeg"), None);
    }
}
