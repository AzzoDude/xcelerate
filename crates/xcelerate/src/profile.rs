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

/// An installed Chrome profile discovered on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeProfile {
    /// Human-readable display name (from `Local State`), falling back to the
    /// directory name.
    pub name: String,
    /// The profile directory name, e.g. `Default` or `Profile 1`.
    pub directory: String,
    /// Absolute path to the profile directory.
    pub path: String,
}

impl ChromeProfile {
    /// Serializes the profile to a JSON object with `name`, `directory`, and
    /// `path` keys.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "directory": self.directory,
            "path": self.path,
        })
    }
}

/// Lists installed Chrome profiles (name + directory + absolute path).
///
/// Reads `<user_data_dir>/Local State` for display names and lists the profile
/// directories that exist on disk, falling back to any subdirectory named
/// `Default` or starting with `Profile `. Returns an empty list when the
/// default user-data directory cannot be determined.
#[must_use]
pub fn list_chrome_profiles() -> Vec<ChromeProfile> {
    let Some(base) = default_chrome_profile_dir() else {
        return Vec::new();
    };

    let names = std::fs::read_to_string(base.join("Local State"))
        .ok()
        .map(|json| parse_profile_names(&json))
        .unwrap_or_default();

    let mut candidates: Vec<String> = names.iter().map(|(dir, _)| dir.clone()).collect();
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
            if !is_dir {
                continue;
            }
            let directory = entry.file_name().to_string_lossy().into_owned();
            if is_profile_directory(&directory) && !candidates.contains(&directory) {
                candidates.push(directory);
            }
        }
    }

    let mut profiles: Vec<ChromeProfile> = candidates
        .into_iter()
        .filter_map(|directory| {
            let path = base.join(&directory);
            if !path.is_dir() {
                return None;
            }
            let name = names
                .iter()
                .find(|(dir, _)| dir == &directory)
                .map(|(_, name)| name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| directory.clone());
            Some(ChromeProfile {
                name,
                directory,
                path: path.to_string_lossy().into_owned(),
            })
        })
        .collect();

    // `Default` first, then alphabetically, for stable output.
    profiles.sort_by(|a, b| {
        (a.directory != "Default", &a.directory).cmp(&(b.directory != "Default", &b.directory))
    });
    profiles
}

/// Absolute path to a named profile directory (`Default`, `Profile 1`, ...).
///
/// Returns `None` when the default user-data directory cannot be determined or
/// the named directory does not exist.
#[must_use]
pub fn chrome_profile_path(directory: &str) -> Option<String> {
    let base = default_chrome_profile_dir()?;
    let path = base.join(directory);
    if path.is_dir() {
        Some(path.to_string_lossy().into_owned())
    } else {
        None
    }
}

/// Whether a directory name looks like a Chrome profile directory.
fn is_profile_directory(name: &str) -> bool {
    name == "Default" || name.starts_with("Profile ")
}

/// Parses the `profile.info_cache` map from a Chrome `Local State` document,
/// returning `(directory, display name)` pairs sorted by directory.
///
/// Malformed input, or input without a `profile.info_cache` object, yields an
/// empty list. Entries missing a usable `name` fall back to the directory name.
pub(crate) fn parse_profile_names(local_state_json: &str) -> Vec<(String, String)> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(local_state_json) else {
        return Vec::new();
    };
    let Some(cache) = value
        .get("profile")
        .and_then(|profile| profile.get("info_cache"))
        .and_then(|cache| cache.as_object())
    else {
        return Vec::new();
    };

    let mut profiles: Vec<(String, String)> = cache
        .iter()
        .map(|(directory, entry)| {
            let name = entry
                .get("name")
                .and_then(|name| name.as_str())
                .filter(|name| !name.is_empty())
                .unwrap_or(directory)
                .to_string();
            (directory.clone(), name)
        })
        .collect();
    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    profiles
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

    #[test]
    fn parses_display_names_from_local_state() {
        let json = r#"{
            "profile": {
                "info_cache": {
                    "Default": { "name": "Personal" },
                    "Profile 1": { "name": "Work" }
                }
            }
        }"#;
        let names = parse_profile_names(json);
        assert_eq!(
            names,
            vec![
                ("Default".to_string(), "Personal".to_string()),
                ("Profile 1".to_string(), "Work".to_string()),
            ]
        );
    }

    #[test]
    fn falls_back_to_directory_when_name_missing() {
        let json = r#"{
            "profile": { "info_cache": { "Profile 2": {}, "Profile 3": { "name": "" } } }
        }"#;
        assert_eq!(
            parse_profile_names(json),
            vec![
                ("Profile 2".to_string(), "Profile 2".to_string()),
                ("Profile 3".to_string(), "Profile 3".to_string()),
            ]
        );
    }

    #[test]
    fn parse_profile_names_is_total() {
        assert!(parse_profile_names("not json").is_empty());
        assert!(parse_profile_names("{}").is_empty());
        assert!(parse_profile_names(r#"{ "profile": {} }"#).is_empty());
    }

    #[test]
    fn recognizes_profile_directories() {
        assert!(is_profile_directory("Default"));
        assert!(is_profile_directory("Profile 1"));
        assert!(is_profile_directory("Profile 42"));
        assert!(!is_profile_directory("System Profile"));
        assert!(!is_profile_directory("Guest Profile"));
        assert!(!is_profile_directory("Local State"));
    }

    #[test]
    fn chrome_profile_serializes_all_fields() {
        let profile = ChromeProfile {
            name: "Personal".into(),
            directory: "Default".into(),
            path: "/home/me/.config/google-chrome/Default".into(),
        };
        let json = profile.to_json();
        assert_eq!(json["name"], serde_json::json!("Personal"));
        assert_eq!(json["directory"], serde_json::json!("Default"));
        assert_eq!(
            json["path"],
            serde_json::json!("/home/me/.config/google-chrome/Default")
        );
    }
}
