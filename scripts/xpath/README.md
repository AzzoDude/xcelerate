# XPath engine regression suite

Node + [jsdom](https://github.com/jsdom/jsdom) tests for the JavaScript XPath
engine that ships in the crate at
`crates/xcelerate/src/page/xpath_engine.js`.

The suite reads the engine from that crate path (`__dirname` + `path.join`) at
runtime, rather than embedding a copy. Because Rust embeds the same file with
`include_str!`, these tests exercise the exact bytes that ship.

## Run

```sh
npm install
npm test
```
