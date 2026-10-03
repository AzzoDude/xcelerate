# Changelog

## 1.0.9

- Initial Dart/Flutter binding for the xcelerate CDP engine.
- Exposes `Browser`, `BrowserConfig`, `Page`, and `Element`, plus the plugin
  bridge (`pluginNames`, `availablePlugins`, `usePlugin`, `loadPlugin`, `plugin`).
- `Browser.launch` is async; `Browser.close` / `Page.close` are exposed as
  `closeBrowser` / `closePage` (the disposer reserves `close`).
