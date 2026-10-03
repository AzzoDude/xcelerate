# Xcelerate Go SDK

Go bindings for the xcelerate Rust CDP engine. Go is not a built-in UniFFI
target, so the sources are generated with the third-party
[`uniffi-bindgen-go`](https://github.com/NordSecurity/uniffi-bindgen-go) generator
(tag `v0.7.1+v0.31.0`, targeting UniFFI 0.31).

## Requirements

- Go 1.21+
- A C toolchain (the generated package uses cgo)
- The `uniffi-bindgen-go` generator:
  `cargo install uniffi-bindgen-go --git https://github.com/NordSecurity/uniffi-bindgen-go --tag v0.7.1+v0.31.0`

## Generate / build

```bash
# from the repository root
python scripts/install_toolchains.py        # installs Go + uniffi-bindgen-go
python scripts/generate_go_bindings.py      # sources + native libs + `go build ./...`
```

or, once generated, inside this directory:

```bash
go build ./...
```

## Layout

```
bindings/go/
  go.mod                        # module github.com/AzzoDude/xcelerate/bindings/go
  xcelerate/
    xcelerate.go                # package xcelerate
    xcelerate.h                 # cgo header
  xcelerate.dll / libxcelerate.so / libxcelerate.dylib
```

The native library must be reachable at run time (next to the built binary, or on
the loader path).

## Usage

```go
package main

import (
    "fmt"

    "github.com/AzzoDude/xcelerate/bindings/go/xcelerate"
)

func main() {
    plugins := []string{"stealth", "human"} // opt into first-party plugins
    browser, err := xcelerate.BrowserLaunch(xcelerate.BrowserConfig{
        Headless: true,
        Detached: true,
        Plugins:  &plugins,
    })
    if err != nil {
        panic(err)
    }
    defer browser.Close()

    page, err := browser.NewPage("https://example.com")
    if err != nil {
        panic(err)
    }

    title, _ := page.Title()
    fmt.Println("title:", title)
    fmt.Println("plugins:", browser.PluginNames())

    human, _ := browser.Plugin("human")
    fmt.Println(human.Invoke("move", `{"x": 320, "y": 240}`))
}
```

Key surface: `BrowserLaunch`, `Browser.NewPage`, `Browser.Close`, and the plugin
bridge (`PluginNames`, `AvailablePlugins`, `UsePlugin`, `LoadPlugin`, `Plugin`,
`PluginHandle.Invoke`). `load_plugin` refuses third-party plugins until the
sandboxed runner ships.

## Publishing

Go modules are distributed through the module proxy, not uploaded. Because the
module lives in `bindings/go`, publish by pushing a **prefixed** tag:

```bash
python scripts/publish_go.py --push
# equivalent to:
git tag bindings/go/v1.0.9
git push origin bindings/go/v1.0.9
# consumers:
go get github.com/AzzoDude/xcelerate/bindings/go/xcelerate@v1.0.9
```

The module is then indexed on
[pkg.go.dev](https://pkg.go.dev/github.com/AzzoDude/xcelerate/bindings/go/xcelerate)
automatically.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
