# Xcelerate Go SDK

Go bindings for the xcelerate Rust CDP engine. Go is not a built-in UniFFI
target; it uses the third-party
[`uniffi-bindgen-go`](https://github.com/NordSecurity/uniffi-bindgen-go) generator.

> **Status: scaffolded, not yet generated.** This machine has neither the Go
> toolchain nor the generator installed, so `scripts/generate_go_bindings.py`
> exists and skips with a hint. Run the steps below to generate the package.

## Requirements

- Go 1.21+
- The `uniffi-bindgen-go` generator:
  `go install github.com/NordSecurity/uniffi-bindgen-go/v2/uniffi-bindgen-go@latest`
- A C toolchain (the generated package uses cgo)

## Generate / build

```bash
# from the repository root
go install github.com/NordSecurity/uniffi-bindgen-go/v2/uniffi-bindgen-go@latest
python scripts/generate_go_bindings.py
```

The script emits a Go package plus the native library:

```
bindings/go/
  go.mod
  *.go                        # generated bindings
  xcelerate.dll / libxcelerate.so / libxcelerate.dylib
```

or, once generated, inside this directory:

```bash
go build ./...
```

## Usage

```go
package main

import (
    "fmt"

    xcelerate "github.com/AzzoDude/xcelerate/go"
)

func main() {
    config := xcelerate.BrowserConfig{
        Headless:  true,
        Stealth:   false,                 // deprecated sugar; prefer Plugins
        Detached:  true,
        Plugins:   []string{"stealth", "human"},
    }
    browser, err := xcelerate.LaunchBrowser(config)
    if err != nil {
        panic(err)
    }
    defer browser.CloseBrowser()

    page, err := browser.NewPage("https://example.com")
    if err != nil {
        panic(err)
    }
    title, _ := page.Title()
    fmt.Println(title)
}
```

Exact symbol names depend on the generator; check the emitted `*.go` files.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
