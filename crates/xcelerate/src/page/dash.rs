//! Minimal DASH (`.mpd`) parsing — enough to download the manifests that real
//! sites serve, with no external tool.
//!
//! This is deliberately a small, dependency-free XML scanner plus a model of the
//! pieces DASH uses to describe segments:
//!
//! - `SegmentTemplate` with `$Number$`, `$Time$` (a `SegmentTimeline`),
//!   `$Bandwidth$` and `$RepresentationID$`.
//! - `SegmentList` with `Initialization` + `SegmentURL` children.
//! - Muxed (one `AdaptationSet` carrying both audio and video) or split tracks.
//!
//! It does **not** decrypt DRM (Widevine/PlayReady) and does not support live
//! (duration-less) manifests or `SegmentBase` byte ranges. Those are reported,
//! not guessed.

/// One downloadable track: an optional init segment plus media segment URLs.
pub(crate) struct Track {
    pub init: Option<String>,
    pub segments: Vec<String>,
    pub kind: Kind,
}

/// Whether a track carries video or audio.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    Video,
    Audio,
}

/// Chooses the best video track and the best audio track from a manifest.
///
/// Returns `(video, audio)`; either may be `None`. A muxed representation (its
/// `mimeType` is not split, or it is the only one) is returned as the video track
/// so a single file is written.
pub(crate) fn plan(
    manifest: &str,
    base_url: &str,
) -> Result<(Option<Track>, Option<Track>), String> {
    let root = parse_xml(manifest).ok_or("could not parse the DASH manifest as XML")?;
    if root.name != "MPD" {
        return Err("the manifest root element is not <MPD>".to_string());
    }
    let presentation_duration =
        attr(&root, "mediaPresentationDuration").and_then(|v| iso8601_seconds(&v));
    let base = join(base_url, attr(&root, "BaseURL").as_deref());

    let period = child(&root, "Period").ok_or("the DASH manifest has no <Period>")?;
    let period_duration = attr(period, "duration")
        .and_then(|v| iso8601_seconds(&v))
        .or(presentation_duration);
    let period_base = join(&base, attr(period, "BaseURL").as_deref());
    let period_template = child(period, "SegmentTemplate").map(parse_template);

    let mut video: Option<(u64, Track)> = None;
    let mut audio: Option<(u64, Track)> = None;

    for adapt in children(period, "AdaptationSet") {
        let adapt_base = join(&period_base, attr(adapt, "BaseURL").as_deref());
        let adapt_template = child(adapt, "SegmentTemplate").map(parse_template);
        let adapt_list = child(adapt, "SegmentList").map(parse_list);
        let adapt_mime = attr(adapt, "mimeType").unwrap_or_default();

        for rep in children(adapt, "Representation") {
            let bandwidth = attr(rep, "bandwidth")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
            let rep_base = join(&adapt_base, attr(rep, "BaseURL").as_deref());
            let mime = attr(rep, "mimeType").unwrap_or_else(|| adapt_mime.clone());
            let rep_template = child(rep, "SegmentTemplate").map(parse_template);
            let rep_list = child(rep, "SegmentList").map(parse_list);

            let template = merge_template(
                &merge_template(&period_template, &adapt_template),
                &rep_template,
            );
            let list = rep_list.or_else(|| adapt_list.clone());
            let rep_id = attr(rep, "id");

            let Some(track) = build_track(
                template.as_ref(),
                list.as_ref(),
                &rep_base,
                rep_id.as_deref(),
                bandwidth,
                period_duration,
            )?
            else {
                continue;
            };

            let kind = classify(
                attr(rep, "contentType")
                    .or_else(|| attr(adapt, "contentType"))
                    .as_deref(),
                &mime,
                Some(track.kind),
            );
            match kind {
                Kind::Video => keep_best(&mut video, bandwidth, track),
                Kind::Audio => keep_best(&mut audio, bandwidth, track),
            }
        }
    }

    Ok((video.map(|(_, t)| t), audio.map(|(_, t)| t)))
}

fn keep_best(slot: &mut Option<(u64, Track)>, bandwidth: u64, track: Track) {
    if slot.as_ref().is_none_or(|(best, _)| bandwidth > *best) {
        *slot = Some((bandwidth, track));
    }
}

/// Decides whether a representation is video, audio, or (muxed) video.
fn classify(content_type: Option<&str>, mime: &str, fallback: Option<Kind>) -> Kind {
    let hint = content_type.unwrap_or("");
    let mime = mime.to_ascii_lowercase();
    if hint.eq_ignore_ascii_case("audio") || mime.starts_with("audio/") {
        return Kind::Audio;
    }
    if hint.eq_ignore_ascii_case("video") || mime.starts_with("video/") {
        return Kind::Video;
    }
    fallback.unwrap_or(Kind::Video)
}

/// Builds a track from a template or a list; `None` when neither applies.
fn build_track(
    template: Option<&Template>,
    list: Option<&SegmentList>,
    base: &str,
    rep_id: Option<&str>,
    bandwidth: u64,
    period_duration: Option<f64>,
) -> Result<Option<Track>, String> {
    if let Some(template) = template {
        return Ok(Some(template.build(
            base,
            rep_id,
            bandwidth,
            period_duration,
        )?));
    }
    if let Some(list) = list {
        let init = list.init.as_deref().map(|u| join(base, Some(u)));
        let segments = list.media.iter().map(|u| join(base, Some(u))).collect();
        return Ok(Some(Track {
            init,
            segments,
            kind: Kind::Video,
        }));
    }
    Ok(None)
}

/// A merged `SegmentTemplate` (period → adaptation → representation).
#[derive(Clone, Default)]
struct Template {
    media: Option<String>,
    initialization: Option<String>,
    start_number: u64,
    timescale: u64,
    duration: Option<u64>,
    timeline: Vec<SEntry>,
}

#[derive(Clone, Copy)]
struct SEntry {
    t: Option<u64>,
    d: u64,
    r: u64,
}

impl Template {
    fn build(
        &self,
        base: &str,
        rep_id: Option<&str>,
        bandwidth: u64,
        period_duration: Option<f64>,
    ) -> Result<Track, String> {
        let timescale = if self.timescale == 0 {
            1
        } else {
            self.timescale
        };
        let media = self
            .media
            .as_deref()
            .ok_or("the SegmentTemplate has no `media` attribute")?;

        let mut segments = Vec::new();
        let mut number = if self.start_number == 0 {
            1
        } else {
            self.start_number
        };

        if !self.timeline.is_empty() {
            let mut time = 0u64;
            for entry in &self.timeline {
                let mut t = entry.t.unwrap_or(time);
                for _ in 0..=entry.r {
                    segments.push(join(
                        base,
                        Some(&expand(media, rep_id, bandwidth, number, t)),
                    ));
                    number += 1;
                    t += entry.d;
                }
                time = t;
            }
        } else if let Some(duration) = self.duration {
            let duration = duration.max(1);
            let total = period_duration.ok_or(
                "the manifest is a live/unknown-length stream (no duration); this downloader only \
                 supports on-demand manifests",
            )?;
            let ticks = (total * timescale as f64).ceil() as u64;
            let count = ticks.div_ceil(duration).max(1);
            for _ in 0..count {
                segments.push(join(
                    base,
                    Some(&expand(media, rep_id, bandwidth, number, 0)),
                ));
                number += 1;
            }
        } else {
            return Err(
                "the SegmentTemplate has neither a SegmentTimeline nor a duration".to_string(),
            );
        }

        let init = self.initialization.as_deref().map(|tmpl| {
            join(
                base,
                Some(&expand(
                    tmpl,
                    rep_id,
                    bandwidth,
                    self.start_number.max(1),
                    0,
                )),
            )
        });

        Ok(Track {
            init,
            segments,
            kind: Kind::Video,
        })
    }
}

fn parse_template(node: &Node) -> Template {
    Template {
        media: attr(node, "media"),
        initialization: attr(node, "initialization"),
        start_number: attr(node, "startNumber")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1),
        timescale: attr(node, "timescale")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1),
        duration: attr(node, "duration").and_then(|v| v.parse().ok()),
        timeline: child(node, "SegmentTimeline")
            .map(|timeline| {
                children(timeline, "S")
                    .iter()
                    .map(|s| SEntry {
                        t: attr(s, "t").and_then(|v| v.parse().ok()),
                        d: attr(s, "d").and_then(|v| v.parse().ok()).unwrap_or(0),
                        r: attr(s, "r").and_then(|v| v.parse().ok()).unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

#[derive(Clone, Default)]
struct SegmentList {
    init: Option<String>,
    media: Vec<String>,
}

fn parse_list(node: &Node) -> SegmentList {
    SegmentList {
        init: child(node, "Initialization")
            .and_then(|i| attr(i, "sourceURL").or_else(|| attr(i, "media"))),
        media: children(node, "SegmentURL")
            .iter()
            .filter_map(|s| attr(s, "media"))
            .collect(),
    }
}

/// Merges a higher-priority template over a lower-priority one, field by field.
fn merge_template(low: &Option<Template>, high: &Option<Template>) -> Option<Template> {
    match (low, high) {
        (None, None) => None,
        (Some(low), None) => Some(low.clone()),
        (None, Some(high)) => Some(high.clone()),
        (Some(low), Some(high)) => Some(Template {
            media: high.media.clone().or_else(|| low.media.clone()),
            initialization: high
                .initialization
                .clone()
                .or_else(|| low.initialization.clone()),
            start_number: if high.start_number == 0 {
                low.start_number
            } else {
                high.start_number
            },
            timescale: if high.timescale == 0 {
                low.timescale
            } else {
                high.timescale
            },
            duration: high.duration.or(low.duration),
            timeline: if high.timeline.is_empty() {
                low.timeline.clone()
            } else {
                high.timeline.clone()
            },
        }),
    }
}

/// Expands a `SegmentTemplate` URL, substituting `$Name$` and `$Name%0Nd$`.
fn expand(template: &str, rep_id: Option<&str>, bandwidth: u64, number: u64, time: u64) -> String {
    let mut out = String::with_capacity(template.len() + 8);
    let mut rest = template;
    while let Some(start) = rest.find('$') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('$') else {
            out.push('$');
            out.push_str(after);
            return out;
        };
        let token = &after[..end];
        out.push_str(&substitute(token, rep_id, bandwidth, number, time));
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

fn substitute(token: &str, rep_id: Option<&str>, bandwidth: u64, number: u64, time: u64) -> String {
    let (name, wide) = match token.split_once('%') {
        Some((name, spec)) => {
            let width: usize = spec
                .trim_end_matches('d')
                .trim_start_matches('0')
                .parse()
                .unwrap_or(0);
            (name, width)
        }
        None => (token, 0),
    };
    let value = match name {
        "RepresentationID" => rep_id.unwrap_or("").to_string(),
        "Bandwidth" => bandwidth.to_string(),
        "Number" => number.to_string(),
        "Time" => time.to_string(),
        _ => return format!("${token}$"),
    };
    if wide > 0 && value.len() < wide {
        format!("{value:0>wide$}")
    } else {
        value
    }
}

/// Joins a possibly-relative URL against a base (a directory or a full URL).
fn join(base: &str, uri: Option<&str>) -> String {
    match uri {
        None => base.to_string(),
        Some("") => base.to_string(),
        Some(uri) => {
            if uri.starts_with("http://") || uri.starts_with("https://") {
                return uri.to_string();
            }
            let base = base.split(['?', '#']).next().unwrap_or(base);
            let trimmed = uri.trim_start_matches('/');
            match base.rfind('/') {
                Some(index) => format!("{}/{}", &base[..index], trimmed),
                None => format!("{base}/{trimmed}"),
            }
        }
    }
}

/// Parses an ISO-8601 duration (`PT1H2M3.5S`) into seconds.
fn iso8601_seconds(value: &str) -> Option<f64> {
    let value = value.trim();
    let rest = value.strip_prefix("PT")?;
    let mut seconds = 0.0f64;
    let mut current = String::new();
    for ch in rest.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            current.push(ch);
            continue;
        }
        let number: f64 = current.parse().ok()?;
        current.clear();
        seconds += match ch {
            'H' => number * 3600.0,
            'M' => number * 60.0,
            'S' => number,
            _ => 0.0,
        };
    }
    Some(seconds)
}

// --- a tiny, dependency-free XML scanner -----------------------------------

struct Node {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

fn attr(node: &Node, name: &str) -> Option<String> {
    node.attrs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.clone())
}

fn child<'a>(node: &'a Node, name: &str) -> Option<&'a Node> {
    node.children
        .iter()
        .find(|child| child.name.eq_ignore_ascii_case(name))
}

fn children<'a>(node: &'a Node, name: &str) -> Vec<&'a Node> {
    node.children
        .iter()
        .filter(|child| child.name.eq_ignore_ascii_case(name))
        .collect()
}

fn parse_xml(src: &str) -> Option<Node> {
    let bytes = src.as_bytes();
    let mut index = 0;
    let mut stack: Vec<Node> = Vec::new();
    let mut root: Option<Node> = None;

    while let Some(open) = src[index..].find('<') {
        let open = index + open;
        // Text between tags is ignored.
        if src[open..].starts_with("<!--") {
            index = src[open..].find("-->").map(|i| open + i + 3)?;
            continue;
        }
        if src[open..].starts_with("<?") || src[open..].starts_with("<!") {
            index = src[open..].find('>').map(|i| open + i + 1)?;
            continue;
        }
        let Some(close) = find_tag_end(bytes, open + 1) else {
            break;
        };
        let inner = &src[open + 1..close];
        if let Some(name) = inner.strip_prefix('/') {
            let node = stack.pop()?;
            if node.name.eq_ignore_ascii_case(name.trim()) {
                push_node(&mut stack, &mut root, node);
            }
        } else {
            let self_closing = inner.ends_with('/');
            let body = inner.trim_end_matches('/').trim_end();
            let (name, attrs) = parse_tag(body);
            let node = Node {
                name,
                attrs,
                children: Vec::new(),
            };
            if self_closing {
                push_node(&mut stack, &mut root, node);
            } else {
                stack.push(node);
            }
        }
        index = close + 1;
    }
    root
}

fn push_node(stack: &mut [Node], root: &mut Option<Node>, node: Node) {
    match stack.last_mut() {
        Some(parent) => parent.children.push(node),
        None => *root = Some(node),
    }
}

/// Finds the `>` that ends a tag, skipping one level of quoted attribute values.
fn find_tag_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut quote: Option<u8> = None;
    for (offset, &byte) in bytes[from..].iter().enumerate() {
        match quote {
            Some(q) if byte == q => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => return Some(from + offset),
            None => {}
        }
    }
    None
}

fn parse_tag(body: &str) -> (String, Vec<(String, String)>) {
    let mut chars = body.char_indices();
    let name_end = chars
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i)
        .unwrap_or(body.len());
    let name = body[..name_end].trim().to_string();
    let mut attrs = Vec::new();
    let mut rest = body[name_end..].trim();
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq].trim().to_string();
        let after = rest[eq + 1..].trim_start();
        let Some(quote) = after.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            break;
        };
        let value_str = &after[1..];
        let Some(end) = value_str.find(quote) else {
            break;
        };
        attrs.push((key, value_str[..end].to_string()));
        rest = value_str[end + 1..].trim();
    }
    (name, attrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"<?xml version="1.0"?>
<MPD mediaPresentationDuration="PT10S" xmlns="urn:mpeg:dash:schema:mpd:2011">
  <Period>
    <AdaptationSet mimeType="video/mp4" contentType="video">
      <SegmentTemplate timescale="1000" duration="2000" startNumber="1"
        initialization="$RepresentationID$/init.mp4"
        media="$RepresentationID$/seg-$Number%05d$.m4s" />
      <Representation id="v0" bandwidth="900000" width="640" height="360" />
      <Representation id="v1" bandwidth="2400000" width="1280" height="720" />
    </AdaptationSet>
    <AdaptationSet mimeType="audio/mp4" contentType="audio">
      <SegmentTemplate timescale="1000" duration="2000" startNumber="1"
        initialization="a/init.mp4" media="a/seg-$Number$.m4s" />
      <Representation id="a0" bandwidth="128000" />
    </AdaptationSet>
  </Period>
</MPD>"#;

    #[test]
    fn plans_video_and_audio_tracks() {
        let (video, audio) = plan(MANIFEST, "https://cdn/x/manifest.mpd").unwrap();
        let video = video.expect("video track");
        assert_eq!(video.init.as_deref(), Some("https://cdn/x/v1/init.mp4"));
        // 10s / 2s per segment = 5 segments.
        assert_eq!(video.segments.len(), 5);
        assert_eq!(video.segments[0], "https://cdn/x/v1/seg-00001.m4s");
        let audio = audio.expect("audio track");
        assert_eq!(audio.segments[0], "https://cdn/x/a/seg-1.m4s");
    }

    #[test]
    fn expands_timeline_and_format_spec() {
        let manifest = r#"<MPD><Period><AdaptationSet mimeType="video/mp4">
            <SegmentTemplate initialization="i.mp4" media="s-$Number$.m4s">
              <SegmentTimeline>
                <S t="0" d="2" r="2"/>
                <S d="1"/>
              </SegmentTimeline>
            </SegmentTemplate>
            <Representation id="v" bandwidth="1"/>
        </AdaptationSet></Period></MPD>"#;
        let (video, _) = plan(manifest, "https://cdn/p.mpd").unwrap();
        let video = video.expect("video");
        assert_eq!(
            video.segments,
            vec![
                "https://cdn/s-1.m4s",
                "https://cdn/s-2.m4s",
                "https://cdn/s-3.m4s",
                "https://cdn/s-4.m4s",
            ]
        );
    }

    #[test]
    fn parses_iso8601_durations() {
        assert_eq!(iso8601_seconds("PT10S"), Some(10.0));
        assert_eq!(iso8601_seconds("PT1M30S"), Some(90.0));
        assert_eq!(iso8601_seconds("PT1H2M3.5S"), Some(3723.5));
        assert_eq!(iso8601_seconds("nope"), None);
    }
}
