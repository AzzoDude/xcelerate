pub mod patcher;
pub mod process;

pub use patcher::BinaryPatcher;
pub use process::{spawn_detached, ProcessRegistry};

pub const STEALTH_JS: &str = r#"
(() => {
    // 1. Hide navigator.webdriver
    const newProto = Object.getPrototypeOf(navigator);
    delete newProto.webdriver;
    Object.defineProperty(navigator, 'webdriver', {
        get: () => undefined
    });

    // 2. Hide Chrome signals (CDC and others)
    // Overriding the getter for the 'chrome' property if it exists
    if (!window.chrome) {
        Object.defineProperty(window, 'chrome', {
            get: () => ({
                runtime: {},
                loadTimes: () => {},
                csi: () => {},
                app: {}
            })
        });
    }

    // 3. Mock plugins (scanners check for empty plugins list)
    if (navigator.plugins.length === 0) {
        Object.defineProperty(navigator, 'plugins', {
            get: () => [
                { name: 'Chrome PDF Viewer', filename: 'internal-pdf-viewer' },
                { name: 'Chromium PDF Viewer', filename: 'internal-pdf-viewer' },
                { name: 'Microsoft Edge PDF Viewer', filename: 'internal-pdf-viewer' },
                { name: 'PDF Viewer', filename: 'internal-pdf-viewer' },
                { name: 'WebKit built-in PDF', filename: 'internal-pdf-viewer' }
            ]
        });
    }

    // 4. Mock languages if missing
    if (!navigator.languages || navigator.languages.length === 0) {
        Object.defineProperty(navigator, 'languages', {
            get: () => ['en-US', 'en']
        });
    }

    // 5. Hide 'cdc_adoQtmxX78vj7j9v9Kw839' which chromedriver might leave behind
    // We already patch the binary for this, but this is a runtime safeguard.
    for (const prop in window) {
        if (prop.startsWith('cdc_') || prop.startsWith('uniffi_')) {
            delete window[prop];
        }
    }
})();
"#;
