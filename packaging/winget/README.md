# Publishing the `xcelerate` CLI to winget

Winget is **Windows only**. The package is `Chaosware.Xcelerate` and installs two
commands: `xcelerate.exe` as `xcelerate`, and `xcelerate-mcp.exe` as
`xcelerate-mcp` (the MCP stdio server). For Linux, see [`../linux`](../linux).

The manifests live in
[`manifests/c/Chaosware/Xcelerate/<version>/`](manifests/c/Chaosware/Xcelerate/1.0.10)
- the same layout [`microsoft/winget-pkgs`][pkgs] uses - and validate cleanly
with `winget validate`. [`wingetcreate`][wc] submits them.

## 1. Build the Windows artifact

From the repository root (PowerShell):

```powershell
cargo build --release -p xcelerate-cli -p xcelerate-mcp
$v = (Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"' |
      Select-Object -First 1).Matches.Groups[1].Value
New-Item -ItemType Directory -Force staging | Out-Null
# The bin target is `xcelerate-cli` (see crates/xcelerate-cli/Cargo.toml);
# ship it as `xcelerate.exe`.
Copy-Item target\release\xcelerate-cli.exe staging\xcelerate.exe
Copy-Item target\release\xcelerate-mcp.exe staging\
Copy-Item LICENSE-MIT,LICENSE-APACHE staging\
Compress-Archive -Path staging\* -DestinationPath "xcelerate-$v-x86_64-pc-windows-msvc.zip" -Force
```

## 2. Publish a GitHub Release the installer URL points at

```powershell
gh release create "v$v" "xcelerate-$v-x86_64-pc-windows-msvc.zip" `
  --title "xcelerate v$v" --notes "See CHANGELOG.md"
```

The resulting, stable URL is:

```
https://github.com/ChaoswareHQ/xcelerate/releases/download/v<version>/xcelerate-<version>-x86_64-pc-windows-msvc.zip
```

## 3. Generate and submit the manifests (recommended)

```powershell
winget install Microsoft.WingetCreate

# First release: answer the interactive prompts
#   PackageIdentifier : Chaosware.Xcelerate
#   Publisher         : Chaosware
#   PackageName       : Xcelerate
#   License           : MIT OR Apache-2.0
#   Moniker           : xcelerate
wingetcreate new "https://github.com/ChaoswareHQ/xcelerate/releases/download/v1.0.10/xcelerate-1.0.10-x86_64-pc-windows-msvc.zip"

# Re-create the manifests non-interactively from this folder's templates:
#   wingetcreate update Chaosware.Xcelerate -u <url> -v <version>

# Open a PR against microsoft/winget-pkgs (needs a GitHub PAT with `public_repo`)
wingetcreate submit --token <PAT> .\manifests\c\Chaosware\Xcelerate\1.0.10
```

`wingetcreate submit` forks [`microsoft/winget-pkgs`][pkgs], writes the manifests
under `manifests/c/Chaosware/Xcelerate/<version>/`, and opens the PR. Microsoft's
CI validates it against the real download; once merged, install with:

```powershell
winget install Chaosware.Xcelerate
xcelerate --version
```

## Doing it by hand

The manifests are already in the `winget-pkgs` layout. For a new version, set
`PackageVersion` and `InstallerUrl`, and update `InstallerSha256` with:

```powershell
(Get-FileHash .\xcelerate-<version>-x86_64-pc-windows-msvc.zip -Algorithm SHA256).Hash
```

Then validate locally and open the PR:

```powershell
winget validate --manifest manifests\c\Chaosware\Xcelerate\<version>
winget install  --manifest manifests\c\Chaosware\Xcelerate\<version>
```

## Localization

The metadata ships in four locales. `en-US` is the default; the others are
`locale` manifests that winget shows to users with a matching locale:

| Locale | File |
| --- | --- |
| English (default) | `Chaosware.Xcelerate.locale.en-US.yaml` |
| Chinese (Simplified) | `Chaosware.Xcelerate.locale.zh-CN.yaml` |
| Japanese | `Chaosware.Xcelerate.locale.ja-JP.yaml` |
| Vietnamese | `Chaosware.Xcelerate.locale.vi-VN.yaml` |

Only the default (`en-US`) manifest carries `Moniker` and `Tags` is fine
anywhere; winget rejects `Moniker` in the other locale files. To add a language,
copy a `locale.*` file, set `PackageLocale` to its BCP-47 tag, and translate the
text fields.

## Notes

- `InstallerType: zip` + `NestedInstallerType: portable` is the right shape here:
  winget extracts the zip and links every `NestedInstallerFiles` entry onto `PATH`
  (`%LOCALAPPDATA%\Microsoft\WinGet\Links\xcelerate.exe` and `…\xcelerate-mcp.exe`).
- Keep the identifier `Chaosware.Xcelerate`; it must be globally unique and is
  what users type (`winget install Chaosware.Xcelerate`). The `Moniker` lets them
  shorten that to `winget install xcelerate`.
- Bump `PackageVersion` every release; winget tracks versions, so publishing the
  same version twice is rejected.
- `xcelerate.exe` vs the `xcelerate` Rust library: the CLI's Cargo bin target is
  named `xcelerate-cli` (see `crates/xcelerate-cli/Cargo.toml`) precisely so it
  no longer collides with the core crate's `xcelerate` lib target (`xcelerate.pdb`
  on Windows). Packaging renames `xcelerate-cli.exe` back to `xcelerate.exe` for
  the zip, so the manifest above and the `xcelerate` command are unchanged.

[wc]: https://github.com/microsoft/winget-create
[pkgs]: https://github.com/microsoft/winget-pkgs
