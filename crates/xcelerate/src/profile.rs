//! Reusing an existing system Chrome profile (capability #7).
//!
//! Implemented as standalone helpers (not tied to `Page`) so they can be called
//! before a browser is launched.

use std::path::{Path, PathBuf};

use crate::error::{XcelerateError, XcelerateResult};

/// Returns the default Chrome user-data directory for the current platform, or
/// `None` when it cannot be determined or does not exist.
///
/// * Windows: `%LOCALAPPDATA%\Google\Chrome\User Data`
/// * macOS: `~/Library/Application Support/Google/Chrome`
/// * Linux: `~/.config/google-chrome`
pub fn default_chrome_profile_dir() -> Option<PathBuf> {
    let candidate = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|base| base.join("Google").join("Chrome").join("User Data"))
    } else if cfg!(target_os = "macos") {
        home_dir().map(|base| {
            base.join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome")
        })
    } else {
        home_dir().map(|base| base.join(".config").join("google-chrome"))
    };

    candidate.filter(|path| path.exists())
}

/// Resolves the user's home directory from `HOME` (or `USERPROFILE` on Windows).
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

/// Recursively copies the Chrome profile at `source` into `dest`, skipping
/// files that would conflict with a running browser (`Singleton*`, `lockfile`)
/// and the large, disposable cache directories.
///
/// `dest` and any missing parents are created. On success the canonical path of
/// `dest` is returned. IO failures are reported as [`XcelerateError::NotFound`].
pub fn reuse_system_profile(source: &str, dest: &str) -> XcelerateResult<String> {
    let source = PathBuf::from(source);
    let dest = PathBuf::from(dest);

    if !source.is_dir() {
        return Err(XcelerateError::NotFound(format!(
            "profile source is not a directory: {}",
            source.display()
        )));
    }

    copy_dir_recursive(&source, &dest)?;

    dest.canonicalize()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| {
            XcelerateError::NotFound(format!(
                "cannot resolve profile destination {}: {error}",
                dest.display()
            ))
        })
}

/// Copies `source` into `dest` recursively, honouring [`is_ignorable`].
fn copy_dir_recursive(source: &Path, dest: &Path) -> XcelerateResult<()> {
    std::fs::create_dir_all(dest).map_err(|error| copy_error(dest, &error))?;

    let entries = std::fs::read_dir(source).map_err(|error| copy_error(source, &error))?;
    for entry in entries {
        let entry = entry.map_err(|error| copy_error(source, &error))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if is_ignorable(&name) {
            continue;
        }

        let from = entry.path();
        let to = dest.join(name.as_ref());
        let file_type = entry
            .file_type()
            .map_err(|error| copy_error(&from, &error))?;
        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|error| copy_error(&from, &error))?;
        }
    }

    Ok(())
}

/// Wraps an IO error with the path it concerns.
fn copy_error(path: &Path, error: &std::io::Error) -> XcelerateError {
    XcelerateError::NotFound(format!(
        "profile copy failed for {}: {error}",
        path.display()
    ))
}

/// Whether a profile entry should be skipped: browser lock/singleton files and
/// the disposable cache directories.
pub(crate) fn is_ignorable(name: &str) -> bool {
    name.starts_with("Singleton")
        || name == "lockfile"
        || matches!(name, "Cache" | "Code Cache" | "GPUCache" | "Crashpad")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_lock_and_singleton_files() {
        assert!(is_ignorable("SingletonLock"));
        assert!(is_ignorable("SingletonCookie"));
        assert!(is_ignorable("SingletonSocket"));
        assert!(is_ignorable("lockfile"));
    }

    #[test]
    fn ignores_cache_directories() {
        assert!(is_ignorable("Cache"));
        assert!(is_ignorable("Code Cache"));
        assert!(is_ignorable("GPUCache"));
        assert!(is_ignorable("Crashpad"));
    }

    #[test]
    fn keeps_real_profile_entries() {
        assert!(!is_ignorable("Default"));
        assert!(!is_ignorable("Local State"));
        assert!(!is_ignorable("Preferences"));
        assert!(!is_ignorable("Cookies"));
    }
}
