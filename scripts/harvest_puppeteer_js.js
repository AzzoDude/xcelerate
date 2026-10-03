// Harvest the public API of Puppeteer classes by introspecting their
// prototype chains at runtime. Prints the same JSON shape the Python harvester
// uses, so it can be merged into adapters/profiles/puppeteer.json.
//
// Puppeteer declares most of the API as *abstract* on the base classes (which
// emits no runtime code) and implements it on the concrete `Cdp*` classes, so
// we union both.
//
// Usage: node harvest_puppeteer_js.js <path-to-puppeteer-core>
//   e.g. node scripts/harvest_puppeteer_js.js ./.probe-puppeteer/node_modules/puppeteer-core

const modulePath = process.argv[2];
if (!modulePath) {
  console.error("usage: node harvest_puppeteer_js.js <path-to-puppeteer-core>");
  process.exit(2);
}

const core = require(modulePath);

const CLASSES = {
  Browser: ["Browser", "CdpBrowser"],
  BrowserContext: ["BrowserContext", "CdpBrowserContext"],
  Page: ["Page", "CdpPage"],
  ElementHandle: ["ElementHandle", "CdpElementHandle"],
  Frame: ["Frame", "CdpFrame"],
};
const STOP_AT = new Set(["EventEmitter", "Object"]);

function collect(cls, into) {
  let proto = cls.prototype;
  while (proto && proto.constructor && !STOP_AT.has(proto.constructor.name)) {
    for (const name of Object.getOwnPropertyNames(proto)) {
      if (name === "constructor" || name.startsWith("_") || into.has(name)) {
        continue;
      }
      const desc = Object.getOwnPropertyDescriptor(proto, name);
      const isProperty = Boolean(desc && (desc.get || desc.set) && !desc.value);
      into.set(name, isProperty);
    }
    proto = Object.getPrototypeOf(proto);
  }
}

const result = { name: "puppeteer", classes: {} };
for (const [logical, impls] of Object.entries(CLASSES)) {
  const found = new Map();
  for (const impl of impls) {
    const cls = core[impl];
    if (typeof cls === "function") {
      collect(cls, found);
    }
  }
  result.classes[logical] = [...found.entries()]
    .map(([name, property]) => ({ name, property }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

process.stdout.write(JSON.stringify(result, null, 2) + "\n");
