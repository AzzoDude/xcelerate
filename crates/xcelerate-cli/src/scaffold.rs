//! `xcelerate plugin new` - scaffold a mod from the bundled template.
//!
//! The template lives in `docs/plugins/template/` and is embedded into the
//! binary, so this works from an installed `xcelerate` with no repo checkout.

use std::fs;
use std::path::PathBuf;

/// Every template file, embedded at build time.
const TEMPLATE: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        include_str!("../../../docs/plugins/template/Cargo.toml"),
    ),
    (
        "plugin.json",
        include_str!("../../../docs/plugins/template/plugin.json"),
    ),
    (
        "README.md",
        include_str!("../../../docs/plugins/template/README.md"),
    ),
    (
        ".gitignore",
        include_str!("../../../docs/plugins/template/.gitignore"),
    ),
    (
        "src/lib.rs",
        include_str!("../../../docs/plugins/template/src/lib.rs"),
    ),
    (
        "src/support.rs",
        include_str!("../../../docs/plugins/template/src/support.rs"),
    ),
];

/// Create a new mod named `name`, returning the directory it was written to.
pub fn new_mod(name: &str, dir: Option<PathBuf>, force: bool) -> std::io::Result<PathBuf> {
    let name = name.trim();
    if name.is_empty() {
        return Err(io_error(
            std::io::ErrorKind::InvalidInput,
            "a mod name is required",
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err(io_error(
            std::io::ErrorKind::InvalidInput,
            "a mod name may only contain [A-Za-z0-9._-]",
        ));
    }

    let title = name.rsplit('.').next().unwrap_or(name);
    let crate_name = name.replace('.', "-");
    let entrypoint = format!("{title}.wasm");
    let dir = dir.unwrap_or_else(|| PathBuf::from(title));

    if dir.exists() && !force {
        return Err(io_error(
            std::io::ErrorKind::AlreadyExists,
            format!(
                "{} already exists (pass --force to write into it anyway)",
                dir.display()
            ),
        ));
    }

    for (relative, contents) in TEMPLATE {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let rendered = contents
            .replace("{{name}}", name)
            .replace("{{crate}}", &crate_name)
            .replace("{{entrypoint}}", &entrypoint)
            .replace("{{title}}", title);
        fs::write(&path, rendered)?;
    }

    // The WIT is the host ABI, not the author's to maintain: write the canonical
    // copy. `xcelerate build` refreshes it, so a plugin can gitignore `wit/`.
    let wit = dir.join("wit").join("plugin.wit");
    if let Some(parent) = wit.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&wit, crate::build::PLUGIN_WIT)?;

    Ok(dir)
}

fn io_error(kind: std::io::ErrorKind, message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(kind, message.into())
}
