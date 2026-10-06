'use strict';
const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { JSDOM } = require('jsdom');

// Read the shipped artifact from the crate. Rust `include_str!`s this exact path,
// so this suite tests the bytes that actually ship rather than a copy.
const XC_XPATH = fs.readFileSync(
  path.join(__dirname, '..', '..', 'crates', 'xcelerate', 'src', 'page', 'xpath_engine.js'),
  'utf8',
);

function make(src) {
  // eslint-disable-next-line no-eval
  return eval('(' + src + ')');
}
const XP = make(XC_XPATH);

function boot(html) {
  const dom = new JSDOM(`<!doctype html><html><body>${html}</body></html>`, {
    runScripts: 'outside-only',
    pretendToBeVisual: true,
  });
  const { window } = dom;
  global.window = window;
  global.document = window.document;
  return window;
}

// Evaluate the engine against a context node.
function run(root, expr, all) {
  return XP.call(root, expr, all);
}

// Native snapshot results, for differential comparison.
function nativeAll(doc, expr) {
  const type = doc.defaultView.XPathResult.ORDERED_NODE_SNAPSHOT_TYPE;
  const res = doc.evaluate(expr, doc, null, type, null);
  const out = [];
  for (let i = 0; i < res.snapshotLength; i++) out.push(res.snapshotItem(i));
  return out;
}

function assertSameNodes(actual, expected, label) {
  assert.strictEqual(actual.length, expected.length, `${label}: length`);
  for (let i = 0; i < expected.length; i++) {
    assert.strictEqual(actual[i], expected[i], `${label}: node at index ${i}`);
  }
}

let passed = 0;
async function ok(name, fn) {
  await fn();
  passed++;
  console.log('  ok -', name);
}

async function main() {
  // 1. Plain light-DOM button.
  await ok('//button finds a plain light-DOM button', () => {
    const w = boot(`<button id="b1">Go</button>`);
    const arr = run(w.document, '//button', true);
    assert.ok(Array.isArray(arr));
    assert.strictEqual(arr.length, 1);
    assert.strictEqual(arr[0].id, 'b1');
    assert.strictEqual(run(w.document, '//button').id, 'b1');
  });

  // 2. Relative path from an element context.
  await ok('.//span does not match the context element itself', () => {
    const w = boot(`<span id="outer"><span id="inner">x</span></span>`);
    const outer = w.document.getElementById('outer');
    const arr = run(outer, './/span', true);
    assert.strictEqual(arr.length, 1);
    assert.strictEqual(arr[0].id, 'inner');
    // A bare relative name step uses the child axis (native semantics).
    assert.strictEqual(run(outer, 'span'), w.document.getElementById('inner'));
  });

  // 3. Predicates.
  await ok('predicates: id, attribute, text, contains, position, and, not', () => {
    const w = boot(`
      <div id="x"><span data-role="save">s</span></div>
      <button>Login</button>
      <a class="top nav-link" href="/a">a</a>
      <ul><li>1</li><li>2</li><li>3</li></ul>
      <input type="text" name="q">
      <p hidden>h</p><p>v</p>
    `);
    assert.strictEqual(run(w.document, "//div[@id='x']", true).length, 1);
    assert.strictEqual(run(w.document, "//*[@data-role='save']", true).length, 1);
    assert.strictEqual(run(w.document, "//button[text()='Login']", true).length, 1);
    assert.strictEqual(run(w.document, "//a[contains(@class,'nav')]", true).length, 1);
    assert.strictEqual(run(w.document, '//li[2]').textContent, '2');
    assert.strictEqual(
      run(w.document, "//input[@type='text' and @name='q']", true).length,
      1,
    );
    const visible = run(w.document, '//p[not(@hidden)]', true);
    assert.strictEqual(visible.length, 1);
    assert.strictEqual(visible[0].textContent, 'v');
    // double-quoted literal and starts-with / normalize-space
    assert.strictEqual(run(w.document, '//button[text()="Login"]', true).length, 1);
    assert.strictEqual(
      run(w.document, "//a[starts-with(text(),'a')]", true).length,
      1,
    );
    assert.strictEqual(
      run(w.document, "//button[normalize-space(text())='Login']", true).length,
      1,
    );
    // and / or combination
    assert.strictEqual(
      run(w.document, "//li[@class='nope' or position()=3]").textContent,
      '3',
    );
  });

  // 4. Shadow DOM.
  await ok('//button pierces an open shadow root', () => {
    const w = boot(`<div id="host"></div>`);
    const host = w.document.getElementById('host');
    host.attachShadow({ mode: 'open' }).innerHTML = `<button class="sh">shadow</button>`;
    const arr = run(w.document, '//button', true);
    assert.strictEqual(arr.length, 1);
    assert.strictEqual(arr[0].textContent, 'shadow');
    // also reachable from the host as context
    assert.strictEqual(run(host, '//button').textContent, 'shadow');
  });

  await ok('nested shadow roots are pierced without duplicates', () => {
    const w = boot(`<div id="outer"></div>`);
    const outer = w.document.getElementById('outer');
    const s1 = outer.attachShadow({ mode: 'open' });
    s1.innerHTML = `<div id="mid"></div><span class="t">one</span>`;
    const s2 = s1.getElementById('mid').attachShadow({ mode: 'open' });
    s2.innerHTML = `<span class="t">two</span><button>deep</button>`;
    const texts = run(w.document, '//span', true).map((e) => e.textContent).sort();
    assert.deepStrictEqual(texts, ['one', 'two']);
    assert.strictEqual(run(w.document, '//button', true).length, 1);
    const everything = run(w.document, '//*', true);
    assert.strictEqual(new Set(everything).size, everything.length, 'no duplicates');
  });

  await ok('closed shadow roots are not entered', () => {
    const w = boot(`<div id="host"></div>`);
    const host = w.document.getElementById('host');
    host.attachShadow({ mode: 'closed' }).innerHTML = `<button>hidden</button>`;
    assert.strictEqual(run(w.document, '//button'), null);
  });

  // 5. Same-origin iframe.
  await ok('//button reaches a same-origin iframe document', () => {
    const w = boot(`<div id="host"><iframe id="f"></iframe></div>`);
    const f = w.document.getElementById('f');
    assert.ok(f.contentDocument, 'iframe doc must exist in jsdom');
    f.contentDocument.body.innerHTML = `<button id="in">in frame</button>`;
    const arr = run(w.document, '//button', true);
    assert.strictEqual(arr.length, 1);
    assert.strictEqual(arr[0].id, 'in');
  });

  await ok('cross-origin frames (null contentDocument) are skipped', () => {
    const w = boot(`<div id="host"><button id="light">L</button><iframe id="f"></iframe></div>`);
    const f = w.document.getElementById('f');
    // Simulate a cross-origin frame: page script cannot read contentDocument.
    Object.defineProperty(f, 'contentDocument', { get: () => null, configurable: true });
    const arr = run(w.document, '//button', true);
    assert.strictEqual(arr.length, 1);
    assert.strictEqual(arr[0].id, 'light');
  });

  // 6. all semantics.
  await ok('all=false returns the first match or null; all=true returns an array', () => {
    const w = boot(`<button id="a">1</button><button id="b">2</button>`);
    assert.strictEqual(run(w.document, '//button').id, 'a');
    assert.strictEqual(run(w.document, '//nope'), null);
    const arr = run(w.document, '//button', true);
    assert.ok(Array.isArray(arr));
    assert.deepStrictEqual(arr.map((e) => e.id), ['a', 'b']);
  });

  // 7. Differential against native on light-DOM-only fixtures.
  await ok('differential: engine equals native document.evaluate on light DOM', () => {
    const w = boot(`
      <div class="c"><span>s1</span></div>
      <div id="d2"><span class="c">s2</span></div>
      <button id="b">B</button>
      <ul><li>1</li><li>2</li><li>3</li></ul>
      <a href="/x">x-link</a><a href="/y">y-link</a>
    `);
    const exprs = [
      '//div',
      '//span',
      "//button[@id='b']",
      "//*[@class='c']",
      '//li[2]',
      "//a[contains(@href,'x')]",
    ];
    for (const expr of exprs) {
      const mine = run(w.document, expr, true);
      const native = nativeAll(w.document, expr);
      assertSameNodes(mine, native, expr);
    }
  });

  // 8. Unsupported syntax throws (caller may fall back).
  await ok('unsupported expressions throw', () => {
    const w = boot(`<a>a</a><b>b</b><div><span>s</span></div>`);
    const bad = [
      '//a|//b', // union
      '//div[count(.//span)>2]', // count()
      '//following-sibling::b', // axis
      '//div/@id', // attribute as step
      '//div[', // unbalanced predicate
    ];
    for (const expr of bad) {
      assert.throws(() => run(w.document, expr, true), expr);
    }
  });

  // 9. text() is existential over ALL direct text-node children.
  await ok('text() matches any direct text child, not just the first', () => {
    const w = boot(`<div id="multi">first<span>mid</span>second</div>`);
    const arr = run(w.document, "//div[text()='second']", true);
    assert.strictEqual(arr.length, 1);
    assert.strictEqual(arr[0].id, 'multi');
    // the first text child still matches too
    assert.strictEqual(run(w.document, "//div[text()='first']", true).length, 1);
    assert.strictEqual(run(w.document, "//*[text()='second']", true).length, 1);
    // 'mid' lives inside a child element, so it is not a direct text child
    assert.strictEqual(run(w.document, "//div[text()='mid']", true).length, 0);
    // contains()/starts-with() are existential over the direct text children
    const w2 = boot(`<div id="d">aa<span>x</span>bb</div>`);
    assert.strictEqual(run(w2.document, "//div[contains(text(),'bb')]", true).length, 1);
    assert.strictEqual(run(w2.document, "//div[starts-with(text(),'aa')]", true).length, 1);
    assert.strictEqual(run(w2.document, "//div[text()]", true).length, 1);
  });

  // 10. Relational operators coerce both operands to numbers (XPath 1.0).
  await ok("relational operators compare numerically: //div[@data-n>'2'] matches 10", () => {
    const w = boot(`<div id="n10" data-n="10"></div><div id="n2" data-n="2"></div><div id="x"></div>`);
    const gt = run(w.document, "//div[@data-n>'2']", true);
    assert.strictEqual(gt.length, 1);
    assert.strictEqual(gt[0].id, 'n10');
    assert.strictEqual(run(w.document, '//div[@data-n>2]', true).length, 1);
    assert.strictEqual(run(w.document, '//div[@data-n<2]', true).length, 0);
    assert.strictEqual(run(w.document, "//div[@data-n<='2']", true)[0].id, 'n2');
    // '=' keeps string semantics when both sides are strings
    assert.strictEqual(run(w.document, "//div[@data-n='10']", true)[0].id, 'n10');
  });

  // 11. Element name tests are case-insensitive for HTML.
  await ok('//DIV matches a <div> (case-insensitive name test)', () => {
    const w = boot(`<div id="case">Case</div><span id="s">s</span>`);
    const arr = run(w.document, '//DIV', true);
    assert.strictEqual(arr.length, 1);
    assert.strictEqual(arr[0].id, 'case');
    assert.strictEqual(run(w.document, "//DIV[@id='case']", true).length, 1);
    assert.strictEqual(run(w.document, '//DiV').id, 'case');
    assert.strictEqual(run(w.document, '//SPAN', true)[0].id, 's');
  });

  // 12. A dangling path separator throws (caller falls back to native).
  await ok('dangling path separators and empty expressions throw', () => {
    const w = boot(`<div id="x"><span>s</span></div>`);
    for (const expr of ['//', '/', 'foo/', 'foo//', '', '   ']) {
      assert.throws(() => run(w.document, expr, true), JSON.stringify(expr));
    }
  });

  console.log(`\n${passed} passing`);
}

main().catch((e) => {
  console.error('FAILED:', e && e.message);
  console.error(e);
  process.exit(1);
});
