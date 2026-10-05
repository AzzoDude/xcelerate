// A minimal end-to-end example for the xcelerate Dart bindings.
//
// Build the native library first (see the package README), then run:
//
//   dart run example/example.dart
import 'package:xcelerate/xcelerate.dart';

Future<void> main() async {
  final browser = await Browser.launch(
    const BrowserConfig(
      headless: true,
      detached: true,
      executablePath: null, // auto-discover Chrome or Edge
      plugins: ['stealth', 'human'], // opt into the built-in plugins
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
