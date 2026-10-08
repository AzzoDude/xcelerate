// A minimal end-to-end example for the xcelerate Dart bindings.
//
// The published package bundles the native library, so this runs as-is. From a
// source checkout, generate the bindings first (see the package README):
//
//   dart run example/example.dart
import 'package:xcelerate/xcelerate.dart';

Future<void> main() async {
  final browser = await Browser.launch(
    const BrowserConfig(
      headless: true,
      detached: true,
      executablePath: null, // auto-discover Chrome or Edge
      plugins: null, // none by default — load one from disk to opt in
    ),
  );

  try {
    final page = await browser.newPage('https://example.com');
    await page.waitForNavigation();
    print('Title: ${await page.title()}');
  } finally {
    await browser.closeBrowser();
    browser.close();
  }
}
