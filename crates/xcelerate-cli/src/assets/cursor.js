// The in-page cursor: a small translucent dot marking where the agent acts,
// plus a click "wave" ripple. Injected on every new document by the CLI.
//
// This is the only in-page injection, and it is deliberately passive: it never
// reads page state and never consumes pointer events.
(function () {
  if (window.__xcelerateCursor) return;
  window.__xcelerateCursor = true;

  var HOST_ID = "__xc_cursor_host";
  var OFF = "-100px";

  // Built as an array (not a template literal) so no backtick can ever sit
  // inside the CSS and terminate the source.
  var css = [
    ":host { all: initial; }",
    ".dot {",
    "  position: absolute;",
    "  left: 0;",
    "  top: 0;",
    "  box-sizing: border-box;",
    "  width: 8px;",
    "  height: 8px;",
    "  margin: -4px 0 0 -4px;",
    "  border-radius: 50%;",
    "  background: radial-gradient(circle at 38% 32%, rgba(255,138,122,0.9) 0%, rgba(255,59,48,0.8) 55%, rgba(224,36,26,0.8) 100%);",
    "  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);",
    "  pointer-events: none;",
    "  opacity: 0;",
    "  transform: translate(var(--x, " + OFF + "), var(--y, " + OFF + ")) scale(var(--s, 1));",
    "  transition: transform .12s ease-out, opacity .35s ease;",
    "}",
    ".dot.down {",
    "  --s: 0.8;",
    "}",
    ".wave {",
    "  position: absolute;",
    "  left: 0;",
    "  top: 0;",
    "  box-sizing: border-box;",
    "  width: 10px;",
    "  height: 10px;",
    "  margin: -5px 0 0 -5px;",
    "  border-radius: 50%;",
    "  background: radial-gradient(circle, rgba(255, 59, 48, 0.45) 0%, rgba(255, 59, 48, 0.22) 45%, rgba(255, 59, 48, 0) 72%);",
    "  pointer-events: none;",
    "  animation: xc-wave .6s ease-out forwards;",
    "}",
    "@keyframes xc-wave {",
    "  from { transform: translate(var(--x, " + OFF + "), var(--y, " + OFF + ")) scale(1); opacity: .9; }",
    "  to { transform: translate(var(--x, " + OFF + "), var(--y, " + OFF + ")) scale(9); opacity: 0; }",
    "}",
  ].join("\n");

  var host = document.createElement("div");
  host.id = HOST_ID;
  host.setAttribute("aria-hidden", "true");
  host.setAttribute("data-xcelerate-cursor", "");
  host.style.cssText =
    "position:fixed;left:0;top:0;width:0;height:0;pointer-events:none;z-index:2147483647;";

  var root = host.attachShadow ? host.attachShadow({ mode: "open" }) : host;

  var style = document.createElement("style");
  style.textContent = css;
  root.appendChild(style);

  var dot = document.createElement("div");
  dot.className = "dot";
  root.appendChild(dot);

  function mount() {
    if (!document.documentElement) return false;
    if (!host.isConnected) document.documentElement.appendChild(host);
    // Stay hidden until the agent actually moves: nothing is shown on load, on
    // reload, or on a new document, so an idle page looks clean.
    return true;
  }
  // At document-start the root may not exist yet; retry once the DOM is ready.
  if (!mount()) {
    document.addEventListener("DOMContentLoaded", mount, { once: true });
  }

  function moveTo(x, y) {
    dot.style.setProperty("--x", x + "px");
    dot.style.setProperty("--y", y + "px");
    dot.style.opacity = "1";
    window.__xcelerateCursorPos = { x: x, y: y };
  }

  function spawnWave(x, y, delay) {
    var wave = document.createElement("div");
    wave.className = "wave";
    wave.style.setProperty("--x", x + "px");
    wave.style.setProperty("--y", y + "px");
    if (delay) wave.style.animationDelay = delay + "s";
    wave.addEventListener("animationend", function () {
      if (wave.parentNode) wave.parentNode.removeChild(wave);
    });
    root.appendChild(wave);
  }

  function pos() {
    return window.__xcelerateCursorPos || null;
  }

  // A soft pulse at the cursor, fired when the agent starts a step - but only
  // once the cursor is actually on screen. A keyboard-only step on a page the
  // agent has not touched stays invisible.
  window.__xceleratePulse = function () {
    var p = pos();
    if (!p) return;
    spawnWave(p.x, p.y, 0);
  };

  function onMove(event) {
    if (!window.__xcelerateDriving) return;
    moveTo(event.clientX, event.clientY);
  }

  function onDown(event) {
    if (!window.__xcelerateDriving) return;
    if (!host.isConnected) mount();
    moveTo(event.clientX, event.clientY);
    dot.classList.add("down");
    spawnWave(event.clientX, event.clientY, 0);
  }

  function onUp() {
    dot.classList.remove("down");
  }

  window.addEventListener("mousemove", onMove, true);
  window.addEventListener("pointermove", onMove, true);
  window.addEventListener("mousedown", onDown, true);
  window.addEventListener("mouseup", onUp, true);

  window.__xcelerateCursorHide = function (hidden) {
    dot.style.display = hidden ? "none" : "block";
  };

  // --- input gate ---------------------------------------------------------
  // A shield the AI raises around its work so a human cannot click, type,
  // scroll or drag the page while the run is driving it. The block runs in the
  // window capture phase (before any page handler) rather than through a
  // covering element: a full-viewport overlay would be treated as an occluder
  // by the indexed snapshot and would hide every interactive element on screen.
  // CDP input is dispatched after the gate is lowered for the step (see
  // `set_driving`).
  var gateOn = false;
  function block(event) {
    if (!gateOn) return;
    event.stopImmediatePropagation();
    event.preventDefault();
  }
  [
    "pointerdown",
    "pointerup",
    "pointermove",
    "mousedown",
    "mouseup",
    "click",
    "dblclick",
    "contextmenu",
    "wheel",
  ].forEach(function (type) {
    window.addEventListener(type, block, { capture: true, passive: false });
  });
  ["keydown", "keypress", "keyup"].forEach(function (type) {
    window.addEventListener(type, block, { capture: true });
  });

  window.__xcelerateGate = function (on) {
    gateOn = !!on;
    return gateOn;
  };
})();
