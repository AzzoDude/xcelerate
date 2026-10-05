//! Built-in device profiles for [`Page::emulate_device`](crate::Page::emulate_device).
//!
//! Like video recording, this API is intentionally **not** exported through
//! UniFFI yet, so adding device profiles never changes the generated bindings'
//! checksums. It is reachable from Rust, the CLI (`--device`, `xcelerate list`),
//! and can be wired into the MCP server the same way.

/// A device profile: viewport, pixel ratio, input mode, and user agent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Device {
    /// Display name, e.g. `iPhone 15 Pro Max`.
    pub name: &'static str,
    /// Lower-case slug accepted by `--device`, e.g. `iphone-15-pro-max`.
    pub slug: &'static str,
    /// Viewport width in CSS pixels.
    pub width: u64,
    /// Viewport height in CSS pixels.
    pub height: i64,
    /// `devicePixelRatio`.
    pub device_scale_factor: f64,
    /// Whether the profile reports a mobile device (`mobile: true`).
    pub mobile: bool,
    /// Whether touch input is emulated.
    pub has_touch: bool,
    /// `navigator.userAgent` override; empty means "leave the browser default".
    pub user_agent: &'static str,
}

const IOS_UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Mobile/15E148 Safari/604.1";
const IPAD_UA: &str = "Mozilla/5.0 (iPad; CPU OS 17_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Mobile/15E148 Safari/604.1";
const PIXEL_UA: &str = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36";
const GALAXY_UA: &str = "Mozilla/5.0 (Linux; Android 14; SM-S911B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36";
const TABLET_UA: &str = "Mozilla/5.0 (Linux; Android 13; SM-X700) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

/// Every built-in device profile, in menu order.
pub fn all() -> &'static [Device] {
    DEVICES
}

/// Find a device by name or slug (case-insensitive; spaces, `_`, and `-` are
/// interchangeable).
pub fn find(query: &str) -> Option<&'static Device> {
    let wanted = normalize(query);
    DEVICES
        .iter()
        .find(|device| normalize(device.name) == wanted || device.slug == wanted)
}

/// Whether the viewport is wider than it is tall.
pub fn is_landscape(device: &Device) -> bool {
    device.width as i64 > device.height
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '_'], "-")
}

static DEVICES: &[Device] = &[
    Device {
        name: "iPhone 15 Pro Max",
        slug: "iphone-15-pro-max",
        width: 430,
        height: 932,
        device_scale_factor: 3.0,
        mobile: true,
        has_touch: true,
        user_agent: IOS_UA,
    },
    Device {
        name: "iPhone 15",
        slug: "iphone-15",
        width: 393,
        height: 852,
        device_scale_factor: 3.0,
        mobile: true,
        has_touch: true,
        user_agent: IOS_UA,
    },
    Device {
        name: "iPhone 13",
        slug: "iphone-13",
        width: 390,
        height: 844,
        device_scale_factor: 3.0,
        mobile: true,
        has_touch: true,
        user_agent: IOS_UA,
    },
    Device {
        name: "iPhone SE",
        slug: "iphone-se",
        width: 375,
        height: 667,
        device_scale_factor: 2.0,
        mobile: true,
        has_touch: true,
        user_agent: IOS_UA,
    },
    Device {
        name: "Pixel 8",
        slug: "pixel-8",
        width: 412,
        height: 915,
        device_scale_factor: 2.625,
        mobile: true,
        has_touch: true,
        user_agent: PIXEL_UA,
    },
    Device {
        name: "Pixel 7",
        slug: "pixel-7",
        width: 412,
        height: 915,
        device_scale_factor: 2.625,
        mobile: true,
        has_touch: true,
        user_agent: PIXEL_UA,
    },
    Device {
        name: "Galaxy S23 Ultra",
        slug: "galaxy-s23-ultra",
        width: 384,
        height: 824,
        device_scale_factor: 3.0,
        mobile: true,
        has_touch: true,
        user_agent: GALAXY_UA,
    },
    Device {
        name: "Galaxy S23",
        slug: "galaxy-s23",
        width: 360,
        height: 780,
        device_scale_factor: 3.0,
        mobile: true,
        has_touch: true,
        user_agent: GALAXY_UA,
    },
    Device {
        name: "iPad Pro 11",
        slug: "ipad-pro-11",
        width: 834,
        height: 1194,
        device_scale_factor: 2.0,
        mobile: true,
        has_touch: true,
        user_agent: IPAD_UA,
    },
    Device {
        name: "iPad Mini",
        slug: "ipad-mini",
        width: 768,
        height: 1024,
        device_scale_factor: 2.0,
        mobile: true,
        has_touch: true,
        user_agent: IPAD_UA,
    },
    Device {
        name: "Galaxy Tab S8",
        slug: "galaxy-tab-s8",
        width: 800,
        height: 1280,
        device_scale_factor: 2.0,
        mobile: true,
        has_touch: true,
        user_agent: TABLET_UA,
    },
    Device {
        name: "Desktop 1080p",
        slug: "desktop",
        width: 1920,
        height: 1080,
        device_scale_factor: 1.0,
        mobile: false,
        has_touch: false,
        user_agent: "",
    },
];
