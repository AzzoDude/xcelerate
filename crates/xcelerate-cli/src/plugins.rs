//! Plugin hosting helpers for the runner: resolving a plugin *name* to a path.
//!
//! The CLI hosts plugins but does not implement them; browsers and native apps
//! live behind the `browser` / `app` / `core` plugins. This module only knows how
//! to find a plugin on disk by bare name.

/// A plugin entry that already exists is used unchanged; otherwise it is treated
/// as a bare name and searched in `$XCELERATE_PLUGIN_DIR`, then the user-global
/// plugin home (`$XCELERATE_HOME`, else `~/.xcl`, `/plugins`), then `./plugins`,
/// then `.`, accepting `<dir>/<name>/` (a plugin directory) or `<dir>/<name>.json`
/// (a manifest). An unresolved name is returned unchanged, so the loader reports
/// the miss with the name the user typed.
pub fn resolve_plugin_name(name: &str) -> String {
    if std::path::Path::new(name).exists() {
        return name.to_string();
    }
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = std::env::var("XCELERATE_PLUGIN_DIR")
        && !dir.is_empty()
    {
        dirs.push(dir.into());
    }
    if let Some(home) = plugin_home() {
        dirs.push(home);
    }
    dirs.push("plugins".into());
    dirs.push(".".into());
    for dir in dirs {
        for candidate in [dir.join(name), dir.join(format!("{name}.json"))] {
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
    }
    name.to_string()
}

/// Resolves each entry of `--plugins` to a concrete path.
pub fn resolve_plugin_names(names: &[String]) -> Vec<String> {
    names.iter().map(|name| resolve_plugin_name(name)).collect()
}

/// The plugin names auto-loaded when installed (`core`, `browser`, `app`), so a
/// script uses the plain verbs without naming the plugin. Capabilities stay
/// default-deny, so this only makes the ops *available*.
pub const STANDARD_PLUGINS: &[&str] = &["core", "browser", "app"];

/// The user-global plugin directory: `$XCELERATE_HOME/plugins` when set,
/// otherwise `~/.xcl/plugins`. One shared home means a plugin built once is
/// importable by name from any project or script.
fn plugin_home() -> Option<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("XCELERATE_HOME")
        && !dir.is_empty()
    {
        return Some(std::path::PathBuf::from(dir).join("plugins"));
    }
    let base = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)?;
    Some(base.join(".xcl").join("plugins"))
}

/// Lists the plugins an operator can load and how to reach them.
pub fn list_plugins() {
    println!("Plugins: none built in (external by design).");
    println!("  Load one with `run --plugins <PATH|NAME>`, or import it in a script.");
    println!("  Scaffold a new one with `xcelerate plugin new <id>`. A bare name resolves from");
    println!("  $XCELERATE_PLUGIN_DIR, then ~/.xcl/plugins, then ./plugins, then .");
}

#[cfg(test)]
mod tests {
    use super::resolve_plugin_name;

    #[test]
    fn existing_paths_and_unknown_names_pass_through() {
        assert_eq!(resolve_plugin_name("Cargo.toml"), "Cargo.toml");
        assert_eq!(
            resolve_plugin_name("no-such-plugin-xyz"),
            "no-such-plugin-xyz"
        );
    }
}
