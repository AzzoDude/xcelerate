// The in-page cursor: a small translucent dot marking where the agent acts,
// plus a click "wave" ripple. Injected on every new document by the CLI.
//
// This is the only remaining in-page injection now that the old HUD (control
// bar + interceptor log) lives in the native overlay window. It is deliberately
// passive: it never reads page state and never consumes pointer events.
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
    "  width: 6px;",
    "  height: 6px;",
    "  margin: -3px 0 0 -3px;",
    "  border-radius: 50%;",
    "  background: rgba(255, 255, 255, 0.8);",
    "  box-shadow: 0 0 6px 2px rgba(255, 255, 255, 0.45),",
    "              0 0 14px 5px rgba(120, 200, 255, 0.35);",
    "  pointer-events: none;",
    "  opacity: 0;",
    "  transform: translate(var(--x, " + OFF + "), var(--y, " + OFF + ")) scale(var(--s, 1));",
    "  transition: transform .12s ease-out, opacity .45s ease;",
    "}",
    ".dot.down {",
    "  --s: 0.6;",
    "}",
    ".wave {",
    "  position: absolute;",
    "  left: 0;",
    "  top: 0;",
    "  width: 6px;",
    "  height: 6px;",
    "  margin: -3px 0 0 -3px;",
    "  border-radius: 50%;",
    "  background: radial-gradient(circle, rgba(255,255,255,0.55) 0%, rgba(120,200,255,0.35) 40%, rgba(120,200,255,0) 70%);",
    "  pointer-events: none;",
    "  animation: xc-wave .6s ease-out forwards;",
    "}",
    "@keyframes xc-wave {",
    "  from { transform: translate(var(--x, " + OFF + "), var(--y, " + OFF + ")) scale(1); opacity: .85; }",
    "  to { transform: translate(var(--x, " + OFF + "), var(--y, " + OFF + ")) scale(16); opacity: 0; }",
    "}",
  ].join("\n");

  var host = document.createElement("div");
  host.id = HOST_ID;
  host.setAttribute("aria-hidden", "true");
  host.setAttribute("data-xcelerate-cursor", "");
  host.style.cssText =
    "position:fixed;inset:0;width:100%;height:100%;pointer-events:none;z-index:2147483647;";

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

  function onMove(event) {
    if (!window.__xcelerateDriving) return;
    moveTo(event.clientX, event.clientY);
  }

  function onDown(event) {
    if (!window.__xcelerateDriving) return;
    if (!host.isConnected) mount();
    moveTo(event.clientX, event.clientY);
    dot.classList.add("down");
    // Two staggered rings read as a single click ripple.
    spawnWave(event.clientX, event.clientY, 0);
    spawnWave(event.clientX, event.clientY, 0.08);
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
  // A full-viewport shield the AI raises around its work so a human cannot
  // click, type, scroll or drag the page while the run is driving it. The host
  // is pointer-events:none, so at rest the shield is inert; when raised it takes
  // pointer events and swallows keyboard events in the capture phase. CDP input
  // is dispatched after the gate is lowered for the step (see `set_driving`).
  var gate = document.createElement("div");
  gate.setAttribute("aria-hidden", "true");
  gate.setAttribute("data-xcelerate-gate", "");
  gate.style.cssText =
    "position:absolute;inset:0;pointer-events:none;cursor:default;";
  root.appendChild(gate);

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
    gate.addEventListener(type, block, { capture: true, passive: false });
  });
  ["keydown", "keypress", "keyup"].forEach(function (type) {
    window.addEventListener(type, block, { capture: true });
  });

  window.__xcelerateGate = function (on) {
    gateOn = !!on;
    gate.style.pointerEvents = gateOn ? "auto" : "none";
    return gateOn;
  };
})();
