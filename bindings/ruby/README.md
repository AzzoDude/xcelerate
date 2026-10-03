# Xcelerate Ruby SDK

Ruby bindings for the xcelerate Rust CDP engine, generated with
[UniFFI](https://mozilla.github.io/uniffi-rs/). Ruby is a built-in UniFFI target.

## Requirements

- Ruby 3.1+
- The native xcelerate library for your platform (`libxcelerate.so`,
  `libxcelerate.dylib`, or `xcelerate.dll`) - built from source by the script below

## Generate / build

```bash
# from the repository root
python scripts/generate_ruby_bindings.py
```

The script runs UniFFI and assembles a gem layout:

```
bindings/ruby/
  lib/xcelerate.rb
  xcelerate.gemspec
  libxcelerate.so / libxcelerate.dylib / xcelerate.dll
```

or, once generated, inside this directory:

```bash
gem build xcelerate.gemspec
gem install ./xcelerate-*.gem
```

## Usage

```ruby
require "xcelerate"

config = Xcelerate::BrowserConfig.new(
  headless: true,
  stealth: false,                    # deprecated sugar; prefer `plugins`
  detached: true,
  executable_path: nil,              # auto-discover Chrome/Edge
  plugins: ["stealth", "human"]      # opt into first-party plugins
)
browser = Xcelerate::Browser.launch(config)
page = browser.new_page("https://example.com")
puts page.title
browser.close
```

The generated bindings are synchronous (UniFFI blocks on the async core).
`load_plugin` refuses third-party plugins until the sandboxed runner ships.

## Publishing

```bash
python scripts/publish_ruby.py          # regenerate + gem build (dry run)
python scripts/publish_ruby.py --push   # gem build + gem push
```

`gem push` authenticates with `~/.gem/credentials` (run `gem signin` once) or the
`GEM_HOST_API_KEY` environment variable. Ship a platform gem per target OS so the
native library is included.

## License

Licensed under either of [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) or
[MIT](https://opensource.org/licenses/MIT), at your option.
