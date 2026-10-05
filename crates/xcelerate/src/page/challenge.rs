//! Bot-check / challenge detection (capability #10).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use crate::error::XcelerateResult;
use crate::page::Page;

/// Anti-bot marker table, as `(needle, vendor, signal)`.
///
/// `vendor` is `None` for generic anti-bot phrasing that does not identify a
/// specific product. Needles are matched against lowercased HTML and URL text.
const MARKERS: &[(&str, Option<&str>, &str)] = &[
    ("g-recaptcha", Some("recaptcha"), "found g-recaptcha markup"),
    (
        "recaptcha",
        Some("recaptcha"),
        "found a reCAPTCHA reference",
    ),
    ("hcaptcha", Some("hcaptcha"), "found hCaptcha markup"),
    (
        "challenges.cloudflare.com",
        Some("turnstile"),
        "found a Cloudflare Turnstile script",
    ),
    (
        "turnstile",
        Some("turnstile"),
        "found a Turnstile reference",
    ),
    (
        "just a moment",
        Some("cloudflare"),
        "shows the Cloudflare \"Just a moment...\" interstitial",
    ),
    (
        "checking your browser",
        Some("cloudflare"),
        "shows a Cloudflare browser check",
    ),
    (
        "cloudflare",
        Some("cloudflare"),
        "found a Cloudflare reference",
    ),
    (
        "px-captcha",
        Some("perimeterx"),
        "found a PerimeterX px-captcha",
    ),
    (
        "perimeterx",
        Some("perimeterx"),
        "found a PerimeterX reference",
    ),
    ("datadome", Some("datadome"), "found a DataDome reference"),
    (
        "pardon our interruption",
        Some("akamai"),
        "shows the Akamai \"Pardon Our Interruption\" interstitial",
    ),
    ("akamai", Some("akamai"), "found an Akamai reference"),
    ("kasada", Some("kasada"), "found a Kasada reference"),
    ("geetest", Some("geetest"), "found a Geetest reference"),
    ("are you a robot", None, "asks \"are you a robot\""),
    ("verify you are human", None, "asks to verify you are human"),
    ("unusual traffic", None, "mentions unusual traffic"),
    ("attention required", None, "shows \"Attention Required\""),
    ("bot check", None, "mentions a bot check"),
    ("pixelscan", None, "found a Pixelscan reference"),
    ("captcha", None, "mentions a captcha"),
];

/// The outcome of scanning a page for bot-check / anti-bot challenges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeReport {
    /// Whether any anti-bot marker was found.
    pub detected: bool,
    /// Canonical vendor ids that were recognized (e.g. `recaptcha`).
    pub vendors: Vec<String>,
    /// Human-readable descriptions of every matched marker.
    pub signals: Vec<String>,
}

impl ChallengeReport {
    /// Serializes the report to a JSON object with `detected`, `vendors`, and
    /// `signals` keys.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "detected": self.detected,
            "vendors": self.vendors,
            "signals": self.signals,
        })
    }
}

/// Scans `html` and `url` (case-insensitively) for known anti-bot markers.
pub(crate) fn analyze_challenge(html: &str, url: &str) -> ChallengeReport {
    let haystack = format!("{}\n{}", html.to_lowercase(), url.to_lowercase());
    let mut vendors: Vec<String> = Vec::new();
    let mut signals: Vec<String> = Vec::new();
    let mut generic = false;

    for &(needle, vendor, signal) in MARKERS {
        if !haystack.contains(needle) {
            continue;
        }
        if !signals.iter().any(|existing| existing.as_str() == signal) {
            signals.push(signal.to_string());
        }
        match vendor {
            Some(vendor) => {
                if !vendors.iter().any(|existing| existing.as_str() == vendor) {
                    vendors.push(vendor.to_string());
                }
            }
            None => generic = true,
        }
    }

    ChallengeReport {
        detected: !vendors.is_empty() || generic,
        vendors,
        signals,
    }
}

impl Page {
    /// Detects whether the page is showing a bot-check / anti-bot challenge.
    ///
    /// Reads the current HTML and URL and classifies them with
    /// [`analyze_challenge`].
    pub async fn detect_challenge(&self) -> XcelerateResult<ChallengeReport> {
        let html = self.content().await?;
        let url = self.url().await?;
        Ok(analyze_challenge(&html, &url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_recaptcha_vendor() {
        let report = analyze_challenge("<div class=\"g-recaptcha\"></div>", "https://example.com");
        assert!(report.detected);
        assert!(report.vendors.iter().any(|v| v == "recaptcha"));
        assert!(!report.signals.is_empty());
    }

    #[test]
    fn plain_page_is_not_flagged() {
        let report = analyze_challenge(
            "<html><body><h1>Welcome</h1></body></html>",
            "https://example.com",
        );
        assert!(!report.detected);
        assert!(report.vendors.is_empty());
        assert!(report.signals.is_empty());
    }

    #[test]
    fn detects_cloudflare_interstitial() {
        let report = analyze_challenge("<title>Just a moment...</title>", "https://example.com");
        assert!(report.detected);
        assert!(report.vendors.iter().any(|v| v == "cloudflare"));
    }

    #[test]
    fn detects_generic_phrase_without_vendor() {
        let report = analyze_challenge("<p>Please verify you are human</p>", "https://example.com");
        assert!(report.detected);
        assert!(report.vendors.is_empty());
        assert!(!report.signals.is_empty());
    }

    #[test]
    fn scans_the_url_too() {
        let report = analyze_challenge(
            "<html></html>",
            "https://challenges.cloudflare.com/turnstile",
        );
        assert!(report.detected);
        assert!(report.vendors.iter().any(|v| v == "turnstile"));
    }

    #[test]
    fn to_json_reports_all_fields() {
        let report = ChallengeReport {
            detected: true,
            vendors: vec!["recaptcha".into()],
            signals: vec!["found g-recaptcha markup".into()],
        };
        let json = report.to_json();
        assert_eq!(json["detected"], serde_json::json!(true));
        assert_eq!(json["vendors"][0], serde_json::json!("recaptcha"));
        assert_eq!(
            json["signals"][0],
            serde_json::json!("found g-recaptcha markup")
        );
    }
}
