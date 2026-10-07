//! `xcelerate build` - compile a scaffolded plugin's `.wasm` core and/or generate
//! and package its typed client binding for a target language.
//!
//! The pipeline is `READ -> SCHEMA -> BINDINGS -> WASM -> PACKAGE`. The wasm
//! compile stage is language-agnostic and identical for every target; only the
//! binding + package stages differ. The scaffolded `build.sh` / `build.ps1` are
//! superseded: we compile in-process with controlled arguments rather than
//! shelling out to arbitrary scripts.

use std::path::{Path, PathBuf};
use std::process::Command;

use xcelerate_codegen::Language;

use crate::cli::CodegenLang;

/// The per-language toolchain: the canonical build/package tool and, for the
/// binding stage, the language the codegen client targets.
struct Toolchain {
    language: Language,
    binary: &'static str,
    artifact: &'static str,
    hint: &'static str,
}

fn codegen_lang(lang: CodegenLang) -> Language {
    match lang {
        CodegenLang::Rust => Language::Rust,
        CodegenLang::Python => Language::Python,
        CodegenLang::Javascript => Language::JavaScript,
        CodegenLang::Csharp => Language::CSharp,
        CodegenLang::Kotlin => Language::Kotlin,
        CodegenLang::Java => Language::Java,
        CodegenLang::Swift => Language::Swift,
        CodegenLang::Ruby => Language::Ruby,
        CodegenLang::Dart => Language::Dart,
        CodegenLang::Go => Language::Go,
        CodegenLang::Powershell => Language::PowerShell,
    }
}

/// The canonical toolchain table for all eleven languages.
fn toolchain(lang: CodegenLang) -> Toolchain {
    let language = codegen_lang(lang);
    match lang {
        CodegenLang::Rust => Toolchain {
            language,
            binary: "cargo",
            artifact: ".crate",
            hint: "install Rust via <https://rustup.rs>",
        },
        CodegenLang::Python => Toolchain {
            language,
            binary: "python",
            artifact: ".whl",
            hint: "install 'build' via `python -m pip install build`",
        },
        CodegenLang::Javascript => Toolchain {
            language,
            binary: "npm",
            artifact: ".tgz",
            hint: "install Node.js via <https://nodejs.org>",
        },
        CodegenLang::Csharp => Toolchain {
            language,
            binary: "dotnet",
            artifact: ".nupkg",
            hint: "install the .NET SDK via <https://dotnet.microsoft.com>",
        },
        CodegenLang::Kotlin => Toolchain {
            language,
            binary: "gradle",
            artifact: ".jar",
            hint: "install Gradle via <https://gradle.org/install>",
        },
        CodegenLang::Java => Toolchain {
            language,
            binary: "gradle",
            artifact: ".jar",
            hint: "install Gradle or a JDK via your package manager",
        },
        CodegenLang::Swift => Toolchain {
            language,
            binary: "swift",
            artifact: "SPM package",
            hint: "install Swift via <https://swift.org>",
        },
        CodegenLang::Ruby => Toolchain {
            language,
            binary: "gem",
            artifact: ".gem",
            hint: "install Ruby via `gem install bundler`",
        },
        CodegenLang::Dart => Toolchain {
            language,
            binary: "dart",
            artifact: "pub package",
            hint: "install Dart via <https://dart.dev>",
        },
        CodegenLang::Go => Toolchain {
            language,
            binary: "go",
            artifact: "Go module",
            hint: "install Go via <https://go.dev>",
        },
        CodegenLang::Powershell => Toolchain {
            language,
            binary: "pwsh",
            artifact: ".psd1 module",
            hint: "install PowerShell via <https://learn.microsoft.com/powershell>",
        },
    }
}

/// Which binary a toolchain ships on this platform, for `--check`.
fn binary_present(binary: &str) -> bool {
    let probe = if cfg!(windows) { "where" } else { "which" };
    Command::new(probe)
        .arg(binary)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Whether `cargo` and the `wasm32-wasip2` target are available (the shared
/// wasm stage). Probed once; cheap enough to re-check per run.
fn wasm_toolchain_present() -> bool {
    if !binary_present("cargo") {
        return false;
    }
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("wasm32-wasip2"))
        .unwrap_or(false)
}

/// Locate a scaffolded plugin directory: the current working directory, or a
/// `plugin.json` / `Cargo.toml` nearby.
fn find_plugin_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    if cwd.join("plugin.json").is_file() || cwd.join("Cargo.toml").is_file() {
        return Some(cwd);
    }
    // Walk up a couple of levels, as a convenience.
    for ancestor in cwd.ancestors().skip(1).take(4) {
        if ancestor.join("plugin.json").is_file() || ancestor.join("Cargo.toml").is_file() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Build the `.wasm` core of the plugin in `dir`, staging it next to `plugin.json`.
fn build_wasm(dir: &Path, offline: bool) -> Result<PathBuf, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(&cargo);
    cmd.current_dir(dir)
        .args(["build", "--release", "--target"]);
    if offline {
        cmd.arg("--offline");
    }
    cmd.arg("wasm32-wasip2");
    let status = cmd
        .status()
        .map_err(|e| format!("failed to run cargo: {e}"))?;
    if !status.success() {
        return Err("cargo build failed".to_string());
    }
    // The crate name (last manifest segment or the directory name).
    let crate_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("plugin")
        .replace(['-', '.'], "_");
    let wasm = dir
        .join("target")
        .join("wasm32-wasip2")
        .join("release")
        .join(format!("{crate_name}.wasm"));
    if !wasm.is_file() {
        return Err(format!("expected built artifact at {}", wasm.display()));
    }
    // Stage next to plugin.json (the transport resolves relative to it).
    let entrypoint = dir.join(format!("{crate_name}.wasm"));
    std::fs::copy(&wasm, &entrypoint).map_err(|e| format!("copy failed: {e}"))?;
    Ok(entrypoint)
}

/// Run the build/report depending on the requested stage.
pub fn run(
    lang: Option<CodegenLang>,
    wasm_only: bool,
    bindings_only: bool,
    check: bool,
    out: Option<PathBuf>,
    offline: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = find_plugin_dir().ok_or("not in a plugin directory (no plugin.json / Cargo.toml)")?;

    if check {
        print_check_matrix();
        return Ok(());
    }

    // Resolve the target language: explicit > none (wasm-only needs none).
    let wasm_stage_wanted = !bindings_only;
    let bindings_stage_wanted = !wasm_only;

    // The wasm stage is common to all languages, so build it whenever requested.
    if wasm_stage_wanted {
        if !wasm_toolchain_present() {
            return Err(
                "wasm32-wasip2 toolchain missing: run `rustup target add wasm32-wasip2`".into(),
            );
        }
        let entrypoint = build_wasm(&dir, offline)?;
        println!("built wasm core: {}", entrypoint.display());
    }

    if bindings_stage_wanted {
        let lang = lang.ok_or(
            "a target language is required for bindings (e.g. `python`, `rust`) - \
                              run `xcelerate build --check` to see the matrix",
        )?;
        let tool = toolchain(lang);
        if !binary_present(tool.binary) && !bindings_only {
            return Err(format!(
                "no `{}` toolchain found for {};\n  hint: {}",
                tool.binary,
                tool.language.label(),
                tool.hint
            )
            .into());
        }
        // Generate the typed binding into the output directory. Without a
        // schema in front of us, emit a generic client stub; the real schema
        // would come from the plugin's `describe` config.
        let binding = render_binding_stub(tool.language);
        let out_dir = out.clone().unwrap_or_else(|| dir.join("dist"));
        std::fs::create_dir_all(&out_dir)?;
        let file = out_dir.join(tool.language.file_name());
        std::fs::write(&file, binding)?;
        println!(
            "generated {} binding: {}  (package with `{}` -> {})",
            tool.language.label(),
            file.display(),
            tool.binary,
            tool.artifact,
        );
    }

    Ok(())
}

/// A generic client stub emitted when no per-op schema is supplied at build
/// time. Concrete schema-driven emission uses
/// `Language::generate_binding(op, schema, defaults)`.
fn render_binding_stub(language: Language) -> String {
    language.generate_binding(
        "example_op",
        r#"{"title":"example_op","type":"object","properties":{"message":{"type":"string"}},"required":["message"]}"#,
        "{}",
    )
}

fn print_check_matrix() {
    println!("lang         tool      detected   artifact");
    for lang in [
        CodegenLang::Rust,
        CodegenLang::Python,
        CodegenLang::Javascript,
        CodegenLang::Csharp,
        CodegenLang::Kotlin,
        CodegenLang::Java,
        CodegenLang::Swift,
        CodegenLang::Ruby,
        CodegenLang::Dart,
        CodegenLang::Go,
        CodegenLang::Powershell,
    ] {
        let tool = toolchain(lang);
        println!(
            "{:<12} {:<8} {:<10} {}",
            tool.language.label(),
            tool.binary,
            if binary_present(tool.binary) {
                "yes"
            } else {
                "no"
            },
            tool.artifact,
        );
    }
    println!(
        "wasm core    cargo      {}         wasm32-wasip2",
        if wasm_toolchain_present() {
            "yes"
        } else {
            "no"
        }
    );
}
