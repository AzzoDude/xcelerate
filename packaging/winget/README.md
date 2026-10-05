# Publishing the `xcelerate` CLI to winget

Winget is **Windows only**. The package is `Chaosware.Xcelerate` and installs
`xcelerate.exe` as the command `xcelerate`. For Linux, see [`../linux`](../linux).

The manifests in this folder are the reference set; [`wingetcreate`][wc] is the
supported way to generate and submit them (it computes the SHA256 for you).

## 1. Build the Windows artifact

From the repository root (PowerShell):

```powershell
cargo build --release -p xcelerate-cli
$v = (Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"' |
      Select-Object -First 1).Matches.Groups[1].Value
New-Item -ItemType Directory -Force staging | Out-Null
Copy-Item target\release\xcelerate.exe staging\
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
wingetcreate submit --token <PAT> .\manifests
```

`wingetcreate submit` forks [`microsoft/winget-pkgs`][pkgs], writes the manifests
under `manifests/c/Chaosware/Xcelerate/<version>/`, and opens the PR. Microsoft's
CI validates it against the real download; once merged, install with:

```powershell
winget install Chaosware.Xcelerate
xcelerate --version
```

## Doing it by hand

Copy the three files in this folder into
`manifests/c/Chaosware/Xcelerate/<version>/`, set `PackageVersion` and
`InstallerUrl`, and replace `REPLACE_WITH_SHA256` with:

```powershell
(Get-FileHash .\xcelerate-<version>-x86_64-pc-windows-msvc.zip -Algorithm SHA256).Hash
```

Then validate locally and open the PR:

```powershell
winget validate --manifest manifests\c\Chaosware\Xcelerate\<version>
winget install  --manifest manifests\c\Chaosware\Xcelerate\<version>
```

## Notes

- `InstallerType: zip` + `NestedInstallerType: portable` is the right shape for a
  single-binary CLI: winget extracts the zip and links `xcelerate.exe` onto
  `PATH` (`%LOCALAPPDATA%\Microsoft\WinGet\Links\xcelerate.exe`).
- Keep the identifier `Chaosware.Xcelerate`; it must be globally unique and is
  what users type (`winget install Chaosware.Xcelerate`). The `Moniker` lets them
  shorten that to `winget install xcelerate`.
- Bump `PackageVersion` every release; winget tracks versions, so publishing the
  same version twice is rejected.
- `xcelerate.exe` vs the `xcelerate` Rust library: both are named `xcelerate`, so
  Cargo warns about a `.pdb` output collision. It is harmless (the outputs are
  `.exe` and `.dll`), but see the note in `crates/xcelerate-cli/Cargo.toml` if you
  later want the warning gone.

[wc]: https://github.com/microsoft/winget-create
[pkgs]: https://github.com/microsoft/winget-pkgs
