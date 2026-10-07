/* xcelerate HUD - a visible cursor and a bottom-center control bar.
 *
 * Injected on every document by `--hud` (alias `--cursor`) or the session
 * `hud` verb. It exists because CDP dispatches real `mousemove`/`mousedown`
 * events but Chromium draws no OS pointer for them, so a run looks like it
 * clicks "air" - and so a human watching can hit Stop.
 *
 * The bottom-left panel is an INTERCEPTOR: it shows the agent's steps and the
 * human's own interactions (clicks, typing, scrolling, navigation) in one live
 * stream, each row tagged AI or YOU.
 *
 * The whole overlay lives inside a SHADOW ROOT. Page CSS loads after this
 * script and would otherwise restyle/clip our buttons (icons drifting up once
 * the page finishes rendering); the shadow boundary keeps the HUD's layout and
 * look private.
 *
 * The bar's buttons publish intent on `window` for the CLI to honour between
 * commands:  __xcelerateStop, __xceleratePause, __xcelerateScreenshot.
 */
(() => {
  if (window.__xcelerateHud) return;
  window.__xcelerateHud = true;

  const HOST_ID = "__xc_host";
  const CSS = `
    :host { all: initial; }

    #__xc_dot, .__xc_wave {
      position: absolute; left: 0; top: 0; pointer-events: none;
      will-change: transform, opacity;
    }
    #__xc_dot {
      width: 6px; height: 6px; margin: -3px 0 0 -3px; border-radius: 50%;
      background: rgba(255,255,255,.92);
      box-shadow: 0 0 6px rgba(255,255,255,.85);
      transform: translate(var(--x, -100px), var(--y, -100px)) scale(var(--s, 1));
      transition: transform .12s ease-out, opacity .45s ease, box-shadow .2s ease;
      opacity: 0;
    }
    #__xc_dot.__xc_down {
      box-shadow: 0 0 16px rgba(255,255,255,1), 0 0 4px rgba(255,255,255,.95);
    }
    .__xc_wave {
      width: 10px; height: 10px; margin: -5px 0 0 -5px; border-radius: 50%;
      background: radial-gradient(circle, rgba(255,255,255,.5) 0%, rgba(255,255,255,.22) 42%, rgba(255,255,255,0) 70%);
      animation: __xc_wave var(--dur, 1.3s) cubic-bezier(.16,.84,.44,1) forwards;
    }
    @keyframes __xc_wave {
      0%   { transform: translate(var(--x), var(--y)) scale(1);   opacity: .95; }
      100% { transform: translate(var(--x), var(--y)) scale(17);  opacity: 0; }
    }

    #__xc_bar {
      position: absolute; left: 50%; bottom: 18px; transform: translateX(-50%);
      display: flex; gap: 2px; align-items: center; padding: 4px;
      background: rgba(18,18,20,.74);
      -webkit-backdrop-filter: blur(12px); backdrop-filter: blur(12px);
      border: 1px solid rgba(255,255,255,.14); border-radius: 999px;
      box-shadow: 0 8px 26px rgba(0,0,0,.5);
      color: #fff; user-select: none; pointer-events: auto; box-sizing: content-box;
      transition: opacity .3s ease, transform .3s cubic-bezier(.2,.9,.25,1.15);
      font: 12px/1 ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif;
    }
    /* While the agent drives the mouse the whole bar - buttons included - is
       click-through and dimmed, so an automated click can never land on a
       control and instead reaches the page underneath. */
    :host([data-driving]) #__xc_bar,
    :host([data-driving]) #__xc_bar button {
      pointer-events: none;
    }
    :host([data-driving]) #__xc_bar { opacity: .3; }
    :host([data-driving]) #__xc_log { pointer-events: none; }
    :host([data-driving]) #__xc_handle { pointer-events: none; opacity: .3; }

    /* Tucked away: the bar slides and fades out, leaving a small tab at the
       bottom center to bring it back. */
    #__xc_bar.__xc_put_away {
      opacity: 0; pointer-events: none;
      transform: translateX(-50%) translateY(16px) scale(.96);
    }
    #__xc_handle {
      position: absolute; left: 50%; bottom: 0;
      display: flex; align-items: center; justify-content: center;
      width: 46px; height: 20px; margin: 0; padding: 0;
      border: 1px solid rgba(255,255,255,.14); border-bottom: 0;
      border-radius: 10px 10px 0 0;
      background: rgba(18,18,20,.74);
      -webkit-backdrop-filter: blur(12px); backdrop-filter: blur(12px);
      color: rgba(255,255,255,.78); cursor: pointer;
      opacity: 0; pointer-events: none;
      transform: translateX(-50%) translateY(10px);
      transition: opacity .25s ease, transform .25s ease, background .2s ease, color .2s ease;
      box-shadow: 0 -6px 18px rgba(0,0,0,.4);
    }
    #__xc_handle.__xc_on {
      opacity: 1; pointer-events: auto;
      transform: translateX(-50%) translateY(0);
    }
    #__xc_handle:hover { background: rgba(32,32,36,.92); color: #fff; }
    #__xc_handle svg { display: block; }
    /* A soft bob so the tab reads as "tap me to bring the bar back". */
    #__xc_handle.__xc_on svg { animation: __xc_bob 2.6s ease-in-out infinite; }
    @keyframes __xc_bob {
      0%, 100% { transform: translateY(1px); }
      50% { transform: translateY(-2px); }
    }

    /* Interceptor: a DevTools-like panel of everything happening on the page -
       the agent's own steps and the human's interactions - captured live.
       Read-only while the agent drives, so it never blocks or is blocked. */
    #__xc_log {
      position: absolute; left: 16px; bottom: 16px; width: 322px;
      display: flex; flex-direction: column;
      background: rgba(18,18,20,.9);
      border: 1px solid rgba(255,255,255,.12); border-radius: 10px;
      box-shadow: 0 12px 32px rgba(0,0,0,.5); overflow: hidden;
      color: #e6e6e8; pointer-events: auto;
      font: 11.5px/1.55 ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif;
    }
    #__xc_log.__xc_collapsed #__xc_log_list { display: none; }
    #__xc_log_head {
      display: flex; align-items: center; gap: 8px; padding: 8px 8px 8px 10px;
      border-bottom: 1px solid rgba(255,255,255,.08);
    }
    #__xc_log_head svg { width: 15px; height: 15px; color: rgba(255,255,255,.55); flex: none; }
    #__xc_log_title { color: rgba(255,255,255,.9); font-weight: 600; font-size: 11.5px; }
    #__xc_log_sub { color: rgba(255,255,255,.3); }
    #__xc_log_head .__xc_spacer { margin-left: auto; }
    #__xc_log_head button {
      width: 22px; height: 22px; display: grid; place-items: center; padding: 0;
      border: 0; border-radius: 6px; background: transparent;
      color: rgba(255,255,255,.5); cursor: pointer;
      transition: background .2s ease, color .2s ease;
    }
    #__xc_log_head button:hover { background: rgba(255,255,255,.1); color: #fff; }
    #__xc_log_head button svg { width: 14px; height: 14px; color: inherit; }
    #__xc_log_count { color: rgba(255,255,255,.4); font-variant-numeric: tabular-nums; }
    #__xc_log_list { max-height: 260px; overflow-y: auto; padding: 4px 5px 6px; }
    #__xc_log_list::-webkit-scrollbar { width: 8px; }
    #__xc_log_list::-webkit-scrollbar-thumb { background: rgba(255,255,255,.14); border-radius: 4px; }
    .__xc_row {
      display: grid; grid-template-columns: 58px minmax(0, 1fr) auto;
      align-items: baseline; gap: 6px; padding: 3px 6px; border-radius: 5px;
      border-left: 2px solid transparent;
    }
    .__xc_row time { color: rgba(255,255,255,.3); font-variant-numeric: tabular-nums; }
    .__xc_row .__xc_right { min-width: 0; }
    .__xc_src {
      display: inline-block; padding: 0 5px; margin-right: 5px; border-radius: 4px;
      font-size: 9.5px; font-weight: 600; letter-spacing: .04em;
      background: rgba(255,255,255,.09); color: rgba(255,255,255,.5);
    }
    .__xc_badge {
      display: inline-block; padding: 0 5px; margin-right: 6px; border-radius: 4px;
      background: rgba(255,255,255,.08); color: rgba(255,255,255,.7);
    }
    /* Human rows carry the same azure accent as the picker, so the two sides of
       the stream read apart at a glance. An error still wins (rule is later). */
    .__xc_row.__xc_human { border-left-color: rgba(125,211,252,.4); }
    .__xc_row.__xc_human .__xc_src { background: rgba(125,211,252,.16); color: #bfe6fb; }
    .__xc_detail { color: rgba(255,255,255,.82); word-break: break-word; }
    .__xc_meta {
      color: rgba(255,255,255,.35); font-variant-numeric: tabular-nums;
      display: inline-flex; align-items: center; gap: 4px; white-space: nowrap;
    }
    .__xc_dot { width: 5px; height: 5px; border-radius: 50%; background: rgba(255,255,255,.45); }
    .__xc_row.__xc_fresh { animation: __xc_flash 2.2s ease forwards; }
    @keyframes __xc_flash {
      0% { background: rgba(255,255,255,.1); }
      100% { background: transparent; }
    }
    .__xc_row.__xc_error { border-left-color: #e06a6a; }
    .__xc_row.__xc_error .__xc_dot { background: #e06a6a; }
    .__xc_row.__xc_error .__xc_detail { color: #e8a9a9; }
    #__xc_log_list:empty::after {
      content: "waiting for activity…"; display: block; padding: 6px;
      color: rgba(255,255,255,.28);
    }

    /* Element picker (F12-style inspect). */
    #__xc_pick {
      position: absolute; display: none; pointer-events: none;
      border: 1px solid #7dd3fc; background: rgba(125,211,252,.16);
      border-radius: 3px; z-index: 2147483646;
    }
    #__xc_pick.__xc_on { display: block; }
    #__xc_pick span {
      position: absolute; left: 0; bottom: 100%; margin-bottom: 3px;
      padding: 2px 6px; border-radius: 5px; white-space: nowrap;
      background: rgba(16,16,19,.96); border: 1px solid rgba(255,255,255,.15);
      color: #bfe6fb; font: 10px/1.5 ui-monospace, Menlo, Consolas, monospace;
    }
    #__xc_bar[data-state="stopped"] {
      border-color: rgba(255,84,72,.8);
      box-shadow: 0 8px 26px rgba(0,0,0,.5), 0 0 18px rgba(255,64,52,.35);
    }
    #__xc_bar button {
      position: relative; width: 28px; height: 28px; margin: 0; padding: 0;
      display: flex; align-items: center; justify-content: center;
      border: 0; border-radius: 50%; background: transparent;
      color: rgba(255,255,255,.82); line-height: 0; cursor: pointer;
      pointer-events: auto; box-sizing: border-box;
      transition: background .15s ease, color .15s ease, transform .1s ease, box-shadow .15s ease;
    }
    #__xc_bar button:hover { background: rgba(255,255,255,.15); color: #fff; }
    #__xc_bar button:active { transform: scale(.9); }
    #__xc_bar button[data-active="true"] { background: rgba(255,255,255,.24); color: #fff; }
    #__xc_bar button[data-kind="stop"]:hover {
      background: rgba(255,64,52,.18);
      color: #ff6a5c;
      box-shadow: inset 0 0 0 1px rgba(255,84,72,.6);
    }
    #__xc_bar button[data-kind="stop"][data-active="true"] {
      background: linear-gradient(180deg, #ff5b4d, #e0312a);
      color: #fff;
      box-shadow: 0 4px 16px rgba(255,52,40,.5);
    }
    /* display:contents drops the wrapper box, so the SVG becomes the button's
       own flex item and is centred by the button's align/justify. */
    #__xc_bar .__xc_icon { display: contents; }
    #__xc_bar .__xc_icon svg { display: block; pointer-events: none; }
    #__xc_bar .__xc_tip {
      position: absolute; bottom: calc(100% + 12px); left: -10px;
      transform: translateY(8px);
      transform-origin: bottom left;
      padding: 6px 11px 8px; border-radius: 10px; white-space: nowrap;
      background: rgba(20,20,24,.96);
      border: 1px solid rgba(255,255,255,.14);
      box-shadow: 0 12px 28px rgba(0,0,0,.45);
      color: #f4f4f5; font-size: 11.5px; font-weight: 500;
      opacity: 0; pointer-events: none;
      transition: opacity .3s ease, transform .3s ease;
    }
    /* Anime-style tail: a 45° corner at the bubble's bottom-left, kept clear of
       the text by the extra bottom padding above. */
    #__xc_bar .__xc_tip::after {
      content: ""; position: absolute; top: 100%; left: 9px;
      width: 10px; height: 10px; margin-top: -6px;
      transform: rotate(45deg);
      border-radius: 0 0 2px 0;
      background: rgba(20,20,24,.96);
      border-right: 1px solid rgba(255,255,255,.14);
      border-bottom: 1px solid rgba(255,255,255,.14);
    }
    #__xc_bar button:hover .__xc_tip {
      opacity: 1;
      transform: translateY(0);
      transition-delay: .12s;
    }
  `;

  // All icons share a 24x24 viewBox with their artwork centred on (12, 12).
  const ICONS = {
    stop: '<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="5.5" y="5.5" width="13" height="13" rx="3"/></svg>',
    pause:
      '<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6.5" y="5.5" width="3.8" height="13" rx="1.6"/><rect x="13.7" y="5.5" width="3.8" height="13" rx="1.6"/></svg>',
    play: '<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M7 5.4v13.2L17.6 12z"/></svg>',
    cursor:
      '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M2.5 12S6 5.8 12 5.8 21.5 12 21.5 12 18 18.2 12 18.2 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="2.7"/></svg>',
    camera:
      '<svg width="16" height="16" viewBox="0 0 256 256" fill="currentColor"><path d="M208,56H180.28L166.65,35.56A8,8,0,0,0,160,32H96a8,8,0,0,0-6.65,3.56L75.71,56H48A24,24,0,0,0,24,80V192a24,24,0,0,0,24,24H208a24,24,0,0,0,24-24V80A24,24,0,0,0,208,56Zm8,136a8,8,0,0,1-8,8H48a8,8,0,0,1-8-8V80a8,8,0,0,1,8-8H80a8,8,0,0,0,6.66-3.56L100.28,48h55.43l13.63,20.44A8,8,0,0,0,176,72h32a8,8,0,0,1,8,8ZM128,88a44,44,0,1,0,44,44A44.05,44.05,0,0,0,128,88Zm0,72a28,28,0,1,1,28-28A28,28,0,0,1,128,160Z"/></svg>',
    picker:
      '<svg width="16" height="16" viewBox="0 0 256 256" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round"><circle cx="128" cy="128" r="70"/><path d="M128 20v34M128 202v34M20 128h34M202 128h34"/></svg>',
    highlight:
      '<svg width="16" height="16" viewBox="0 0 256 256" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"><path d="M40 84V56a16 16 0 0 1 16-16h28M172 40h28a16 16 0 0 1 16 16v28M216 172v28a16 16 0 0 1-16 16h-28M84 216H56a16 16 0 0 1-16-16v-28"/><circle cx="128" cy="128" r="26"/></svg>',
    trash:
      '<svg width="16" height="16" viewBox="0 0 256 256" fill="none" stroke="currentColor" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"><path d="M40 72h176M96 72V52a12 12 0 0 1 12-12h40a12 12 0 0 1 12 12v20M64 72l8 132a16 16 0 0 0 16 15h80a16 16 0 0 0 16-15l8-132"/></svg>',
    chevron:
      '<svg width="14" height="14" viewBox="0 0 256 256" fill="none" stroke="currentColor" stroke-width="22" stroke-linecap="round" stroke-linejoin="round"><path d="M60 100l68 64 68-64"/></svg>',
    up: '<svg width="14" height="14" viewBox="0 0 256 256" fill="none" stroke="currentColor" stroke-width="22" stroke-linecap="round" stroke-linejoin="round"><path d="M60 156l68-64 68 64"/></svg>',
    close:
      '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M6.8 6.8l10.4 10.4M17.2 6.8L6.8 17.2"/></svg>',
  };

  // Robot logo for the activity log (Phosphor "robot", 256 viewBox).
  const ROBOT =
    '<svg viewBox="0 0 256 256" fill="currentColor"><path d="M200,48H136V16a8,8,0,0,0-16,0V48H56A32,32,0,0,0,24,80V192a32,32,0,0,0,32,32H200a32,32,0,0,0,32-32V80A32,32,0,0,0,200,48Zm16,144a16,16,0,0,1-16,16H56a16,16,0,0,1-16-16V80A16,16,0,0,1,56,64H200a16,16,0,0,1,16,16Zm-52-56H92a28,28,0,0,0,0,56h72a28,28,0,0,0,0-56Zm-24,16v24H116V152ZM80,164a12,12,0,0,1,12-12h8v24H92A12,12,0,0,1,80,164Zm84,12h-8V152h8a12,12,0,0,1,0,24ZM72,108a12,12,0,1,1,12,12A12,12,0,0,1,72,108Zm88,0a12,12,0,1,1,12,12A12,12,0,0,1,160,108Z"/></svg>';

  let root;
  let host;
  let dot;
  let bar;
  let handle;
  let logList;
  let logCount = 0;
  let pickerOn = false;
  let pickerBox;
  let highlightStyle;
  let lastX = -100;
  let lastY = -100;

  const place = (x, y) => {
    dot.style.setProperty("--x", x + "px");
    dot.style.setProperty("--y", y + "px");
  };

  const spawnWave = (slow) => {
    const wave = document.createElement("div");
    wave.className = "__xc_wave";
    if (slow) wave.style.setProperty("--dur", "1.7s");
    wave.style.setProperty("--x", lastX + "px");
    wave.style.setProperty("--y", lastY + "px");
    root.appendChild(wave);
    wave.addEventListener("animationend", () => wave.remove(), { once: true });
  };

  const installCursor = () => {
    dot = document.createElement("div");
    dot.id = "__xc_dot";
    root.appendChild(dot);
    place(lastX, lastY);
    dot.style.display = window.__xcelerateCursorHidden ? "none" : "block";

    const onMove = (event) => {
      // Only the agent's pointer drives the dot. A human moving the mouse over the
      // page must not drag it around; the HUD is for watching the agent, and the
      // bar itself stays usable.
      if (!window.__xcelerateDriving) return;
      lastX = event.clientX;
      lastY = event.clientY;
      window.__xcelerateCursorPos = { x: lastX, y: lastY };
      place(lastX, lastY);
      dot.style.opacity = "1";
    };
    window.addEventListener("mousemove", onMove, true);
    window.addEventListener("pointermove", onMove, true);

    window.addEventListener(
      "mousedown",
      () => {
        // Only the agent's clicks throw a wave; a human clicking the page does not.
        if (!window.__xcelerateDriving) return;
        dot.classList.add("__xc_down");
        dot.style.setProperty("--s", ".5");
        spawnWave(false);
        spawnWave(true);
      },
      true,
    );
    window.addEventListener(
      "mouseup",
      () => {
        dot.classList.remove("__xc_down");
        dot.style.setProperty("--s", "1");
      },
      true,
    );
  };

  const installBar = () => {
    bar = document.createElement("div");
    bar.id = "__xc_bar";

    const make = (kind, iconSvg, tip, onActivate) => {
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.kind = kind;
      const icon = document.createElement("span");
      icon.className = "__xc_icon";
      icon.innerHTML = iconSvg;
      const label = document.createElement("span");
      label.className = "__xc_tip";
      label.textContent = tip;
      button.appendChild(icon);
      button.appendChild(label);
      // Never take focus on press: the human using the bar must not steal the
      // caret from an input the agent is (or will be) typing into.
      button.addEventListener("mousedown", (event) => event.preventDefault(), true);
      button.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        onActivate(button, icon);
      });
      bar.appendChild(button);
      return { button, icon };
    };

    // Stop - the primary control: signal the CLI to end the session.
    make("stop", ICONS.stop, "Stop the agent", (button) => {
      window.__xcelerateStop = true;
      button.dataset.active = "true";
      bar.dataset.state = "stopped";
    });

    // Pause / resume.
    const pause = make("pause", ICONS.pause, "Pause / resume", (button, icon) => {
      window.__xceleratePause = !window.__xceleratePause;
      button.dataset.active = String(!!window.__xceleratePause);
      icon.innerHTML = window.__xceleratePause ? ICONS.play : ICONS.pause;
    });
    pause.button.dataset.active = "false";

    // Toggle the cursor dot.
    make("cursor", ICONS.cursor, "Show / hide the cursor", (button) => {
      window.__xcelerateCursorHidden = !window.__xcelerateCursorHidden;
      if (dot) dot.style.display = window.__xcelerateCursorHidden ? "none" : "block";
      button.dataset.active = String(!!window.__xcelerateCursorHidden);
    });

    // Request a screenshot from the CLI.
    make("shot", ICONS.camera, "Request a screenshot", (button) => {
      window.__xcelerateScreenshot = true;
      button.dataset.active = "true";
      setTimeout(() => (button.dataset.active = "false"), 400);
    });

    // F12-style element picker: hover to outline, click to copy a selector.
    make("pick", ICONS.picker, "Pick an element (copies a selector)", (button) => {
      setPicker(!pickerOn);
      button.dataset.active = String(pickerOn);
    });

    // Outline every clickable element on the page.
    make("highlight", ICONS.highlight, "Highlight clickable elements", (button) => {
      const on = button.dataset.active !== "true";
      setHighlight(on);
      button.dataset.active = String(on);
    });

    // Show / hide the inspector panel.
    make("inspector", ROBOT, "Show / hide the interceptor", (button) => {
      const panel = root.getElementById("__xc_log");
      if (!panel) return;
      const hidden = panel.style.display === "none";
      panel.style.display = hidden ? "" : "none";
      button.dataset.active = String(!hidden);
    });

    // Tuck the bar away; the cursor and the interceptor stay. A small tab at the
    // bottom center brings it back.
    make("close", ICONS.close, "Hide this bar", () => setBar(false));

    root.appendChild(bar);
    setBar(window.__xcelerateHudBar !== false);
  };

  // Shows or tucks away the bar, keeping the reopen tab in step. While the agent
  // drives the tab is click-through (see the :host([data-driving]) rule), so an
  // automated click can never land on it.
  const setBar = (visible) => {
    if (!bar) return;
    window.__xcelerateHudBar = visible;
    bar.classList.toggle("__xc_put_away", !visible);
    if (handle) handle.classList.toggle("__xc_on", !visible);
  };

  // The reopen tab: a small arrow at the bottom center that animates the bar back.
  const installHandle = () => {
    if (handle && handle.isConnected) return;
    handle = document.createElement("button");
    handle.type = "button";
    handle.id = "__xc_handle";
    handle.setAttribute("aria-label", "Show controls");
    handle.innerHTML = ICONS.up;
    // Same rule as the bar: pressing must not steal focus from the agent's input.
    handle.addEventListener("mousedown", (event) => event.preventDefault(), true);
    handle.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      if (!bar || !bar.isConnected) {
        installBar();
        bar.classList.add("__xc_put_away");
        // Force a reflow so the tucked state is committed before we animate in.
        void bar.offsetWidth;
        handle.classList.add("__xc_on");
      }
      setBar(true);
    });
    root.appendChild(handle);
    if (window.__xcelerateHudBar === false) handle.classList.add("__xc_on");
  };

  const installLog = () => {
    const panel = document.createElement("div");
    panel.id = "__xc_log";

    const list = document.createElement("div");
    list.id = "__xc_log_list";

    const head = document.createElement("div");
    head.id = "__xc_log_head";
    head.innerHTML =
      ROBOT +
      '<span id="__xc_log_title">Interceptor</span>' +
      '<span id="__xc_log_sub">live</span><span class="__xc_spacer"></span>';

    const count = document.createElement("span");
    count.id = "__xc_log_count";
    count.textContent = "0";
    head.appendChild(count);

    const clear = document.createElement("button");
    clear.type = "button";
    clear.title = "Clear the inspector";
    clear.innerHTML = ICONS.trash;
    clear.addEventListener("click", () => {
      list.textContent = "";
      logCount = 0;
      count.textContent = "0";
    });
    head.appendChild(clear);

    const collapse = document.createElement("button");
    collapse.type = "button";
    collapse.title = "Collapse / expand";
    collapse.innerHTML = ICONS.chevron;
    collapse.addEventListener("click", () => {
      const folded = panel.classList.toggle("__xc_collapsed");
      collapse.style.transform = folded ? "rotate(180deg)" : "";
    });
    head.appendChild(collapse);

    panel.appendChild(head);
    panel.appendChild(list);
    root.appendChild(panel);
    logList = list;
  };

  // Append one structured record to the inspector (called by the CLI per step).
  // --- element picker + highlighter (F12-style helpers) ----------------------
  const cssPath = (el) => {
    if (!el || el.nodeType !== 1) return "";
    if (el.id) return "#" + el.id;
    const parts = [];
    let node = el;
    while (node && node.nodeType === 1 && parts.length < 4) {
      if (node.id) {
        parts.unshift("#" + node.id);
        break;
      }
      let part = node.tagName.toLowerCase();
      const cls = (node.getAttribute("class") || "")
        .trim()
        .split(/\s+/)
        .filter(Boolean)
        .slice(0, 2);
      if (cls.length) part += "." + cls.join(".");
      const parent = node.parentElement;
      if (parent) {
        const same = [...parent.children].filter((c) => c.tagName === node.tagName);
        if (same.length > 1) part += `:nth-of-type(${same.indexOf(node) + 1})`;
      }
      parts.unshift(part);
      node = parent;
    }
    return parts.join(" > ");
  };

  // Short human-readable label for an element: its selector plus, when it has
  // one, a trimmed snippet of its visible or accessible text.
  const describeTarget = (el) => {
    if (!el || el.nodeType !== 1) return "";
    const path = cssPath(el);
    const raw =
      (el.getAttribute ? el.getAttribute("aria-label") : "") ||
      el.innerText ||
      el.textContent ||
      "";
    const text = String(raw).replace(/\s+/g, " ").trim().slice(0, 32);
    return text ? `${path} "${text}"` : path;
  };

  const ensurePickerBox = () => {
    if (pickerBox && pickerBox.isConnected) return pickerBox;
    pickerBox = document.createElement("div");
    pickerBox.id = "__xc_pick";
    pickerBox.appendChild(document.createElement("span"));
    root.appendChild(pickerBox);
    return pickerBox;
  };

  const onPickerMove = (event) => {
    const el = document.elementFromPoint(event.clientX, event.clientY);
    const box = ensurePickerBox();
    if (!el || (host && host.contains(el))) {
      box.classList.remove("__xc_on");
      return;
    }
    const rect = el.getBoundingClientRect();
    box.style.left = rect.left + "px";
    box.style.top = rect.top + "px";
    box.style.width = rect.width + "px";
    box.style.height = rect.height + "px";
    box.querySelector("span").textContent = cssPath(el);
    box.classList.add("__xc_on");
  };

  const onPickerClick = (event) => {
    // Never swallow the agent's own clicks.
    if (!pickerOn || window.__xcelerateDriving) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    const el = document.elementFromPoint(event.clientX, event.clientY);
    const selector = cssPath(el);
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(selector);
      }
    } catch (_) {}
    window.__xcelerateLog({ action: "pick", detail: selector, status: "ok", source: "human" });
    setPicker(false);
  };

  const setPicker = (on) => {
    pickerOn = !!on;
    if (pickerOn) {
      window.addEventListener("mousemove", onPickerMove, true);
      window.addEventListener("mousedown", onPickerClick, true);
      document.documentElement.style.cursor = "crosshair";
    } else {
      window.removeEventListener("mousemove", onPickerMove, true);
      window.removeEventListener("mousedown", onPickerClick, true);
      document.documentElement.style.cursor = "";
      if (pickerBox) pickerBox.classList.remove("__xc_on");
    }
  };

  const setHighlight = (on) => {
    if (on) {
      if (!highlightStyle) {
        highlightStyle = document.createElement("style");
        highlightStyle.id = "__xc_highlight";
        highlightStyle.textContent =
          'a,button,input,select,textarea,summary,[role="button"],[role="link"],[onclick]' +
          '{outline:2px solid rgba(125,211,252,.9) !important;outline-offset:1px !important;' +
          'background-color:rgba(125,211,252,.1) !important;}';
        (document.head || document.documentElement).appendChild(highlightStyle);
      }
      highlightStyle.disabled = false;
    } else if (highlightStyle) {
      highlightStyle.disabled = true;
    }
  };

  // Human-side interception: records the person's own clicks, typing, scrolling
  // and navigation in the same stream as the agent's steps, tagged `human`.
  // Events are skipped while the agent drives - so an automated step is never
  // double-counted alongside the agent's own row - and when they originate
  // inside the HUD itself.
  const installIntercept = () => {
    if (window.__xcelerateIntercept) return;
    window.__xcelerateIntercept = true;

    const ACTION_KEYS = ["Enter", "Escape", "Tab"];

    const fromHud = (event) => {
      if (!host) return false;
      if (typeof event.composedPath === "function" && event.composedPath().includes(host)) {
        return true;
      }
      return host.contains(event.target);
    };

    // A page dispatching its own events reports isTrusted=false; only real input
    // (from a person or from CDP) is trusted, so scripted noise never shows up.
    const interesting = (event) =>
      event.isTrusted && !window.__xcelerateDriving && !fromHud(event);

    const record = (action, detail) => {
      window.__xcelerateLog({ action, detail, status: "ok", source: "human" });
    };

    window.addEventListener(
      "click",
      (event) => {
        if (!interesting(event)) return;
        record("click", describeTarget(event.target));
      },
      true,
    );

    window.addEventListener(
      "dblclick",
      (event) => {
        if (!interesting(event)) return;
        record("dblclick", describeTarget(event.target));
      },
      true,
    );

    window.addEventListener(
      "keydown",
      (event) => {
        if (!interesting(event)) return;
        const combo = (event.ctrlKey || event.metaKey) && event.key.length === 1;
        if (!combo && ACTION_KEYS.indexOf(event.key) === -1) return;
        record("key", `pressed ${event.key}`);
      },
      true,
    );

    // Typing arrives one event per character; coalesce a burst into one row.
    let typedCount = 0;
    let typedTarget = null;
    let typedTimer = null;
    window.addEventListener(
      "input",
      (event) => {
        if (!interesting(event)) return;
        typedCount += 1;
        typedTarget = event.target;
        if (typedTimer) clearTimeout(typedTimer);
        typedTimer = setTimeout(() => {
          typedTimer = null;
          const count = typedCount;
          const target = typedTarget;
          typedCount = 0;
          if (count > 0) record("type", `${count} chars into ${cssPath(target)}`);
        }, 500);
      },
      true,
    );

    window.addEventListener(
      "change",
      (event) => {
        if (!interesting(event)) return;
        const el = event.target;
        const tag = el.tagName ? el.tagName.toLowerCase() : "";
        const type = (el.type || "").toLowerCase();
        const toggle = type === "checkbox" || type === "radio";
        const pick = ["file", "range", "color", "date", "time", "month", "week"].indexOf(type) !== -1;
        if (tag !== "select" && !toggle && !pick) return;
        const value = toggle ? (el.checked ? "checked" : "unchecked") : String(el.value || "").slice(0, 40);
        record("change", value ? `${cssPath(el)} = ${value}` : cssPath(el));
      },
      true,
    );

    window.addEventListener(
      "submit",
      (event) => {
        if (!interesting(event)) return;
        record("submit", cssPath(event.target));
      },
      true,
    );

    // Scrolling fires continuously; report where it settled.
    let scrollTimer = null;
    window.addEventListener(
      "scroll",
      () => {
        if (window.__xcelerateDriving) return;
        if (scrollTimer) clearTimeout(scrollTimer);
        scrollTimer = setTimeout(() => {
          scrollTimer = null;
          record("scroll", `to ${Math.round(window.scrollY)}px`);
        }, 400);
      },
      true,
    );

    for (const type of ["popstate", "hashchange"]) {
      window.addEventListener(type, () => {
        if (window.__xcelerateDriving) return;
        record("navigate", `${location.pathname}${location.hash}`);
      });
    }
  };

  const install = () => {
    if (!document.documentElement) return;

    let existing = document.getElementById(HOST_ID);
    if (!existing) {
      existing = document.createElement("div");
      existing.id = HOST_ID;
      // Hidden from the page's accessibility tree so xcelerate's own snapshots
      // never offer the HUD to the agent as something to click.
      existing.setAttribute("aria-hidden", "true");
      existing.setAttribute("data-xcelerate-hud", "");
      // Fixed, viewport-sized, click-through: page CSS cannot reach inside the
      // shadow tree, and the host itself never intercepts the page's own clicks.
      existing.style.cssText =
        "position:fixed;inset:0;width:100%;height:100%;pointer-events:none;z-index:2147483647;";
      document.documentElement.appendChild(existing);
    }
    host = existing;
    root = host.shadowRoot || host.attachShadow({ mode: "open" });

    if (!root.querySelector("style")) {
      const style = document.createElement("style");
      style.textContent = CSS;
      root.appendChild(style);
    }

    if (!root.querySelector("#__xc_dot")) installCursor();
    if (!root.querySelector("#__xc_log")) installLog();
    installHandle();
    if (!root.querySelector("#__xc_bar") && window.__xcelerateHudBar !== false) installBar();
    installIntercept();
  };

  // While the agent is dispatching pointer input, the bar becomes click-through
  // and dimmed: an automated click can never land on a control, and any click the
  // agent makes there passes straight to the page. It returns to normal when the
  // CLI finishes the action, so the human can use it again (and press Stop).
  window.__xcelerateSetDriving = (on) => {
    window.__xcelerateDriving = !!on;
    if (host) host.toggleAttribute("data-driving", !!on);
  };

  // Append one structured record to the interceptor (called by the CLI per step
  // and by the page-side human handlers above).
  // Accepts an object { action, detail, status, ms, source }; a bare string is
  // treated as a plain detail line, keeping it backwards compatible.
  window.__xcelerateLog = (payload) => {
    if (!logList) return false;
    const entry = typeof payload === "string" ? { detail: payload } : payload || {};
    const action = String(entry.action || "step");
    const detail = String(entry.detail || "");
    const source = entry.source === "human" ? "human" : "agent";
    const isError = entry.status === "error";
    const ms = Number.isFinite(entry.ms) ? entry.ms : null;

    const now = new Date();
    const pad = (n) => String(n).padStart(2, "0");
    const stamp = `${pad(now.getHours())}:${pad(now.getMinutes())}:${pad(now.getSeconds())}`;

    const row = document.createElement("div");
    row.className = isError ? "__xc_row __xc_error __xc_fresh" : "__xc_row __xc_fresh";
    if (source === "human") row.classList.add("__xc_human");

    const time = document.createElement("time");
    time.textContent = stamp;

    const right = document.createElement("div");
    right.className = "__xc_right";
    const src = document.createElement("span");
    src.className = "__xc_src";
    src.textContent = source === "human" ? "YOU" : "AI";
    const badge = document.createElement("span");
    badge.className = "__xc_badge";
    badge.textContent = action;
    const text = document.createElement("span");
    text.className = "__xc_detail";
    text.textContent = detail;
    right.appendChild(src);
    right.appendChild(badge);
    right.appendChild(text);

    const meta = document.createElement("span");
    meta.className = "__xc_meta";
    const dot = document.createElement("i");
    dot.className = "__xc_dot";
    meta.appendChild(dot);
    if (ms !== null) meta.appendChild(document.createTextNode(`${ms}ms`));

    row.appendChild(time);
    row.appendChild(right);
    row.appendChild(meta);

    logList.prepend(row);
    while (logList.childElementCount > 80) logList.lastElementChild.remove();
    setTimeout(() => row.classList.remove("__xc_fresh"), 1300);

    logCount += 1;
    const badgeCount = root.getElementById("__xc_log_count");
    if (badgeCount) badgeCount.textContent = String(logCount);
    return true;
  };

  // Fire a wave at the last known position (for demos / screenshots).
  window.__xcelerateRipple = () => {
    spawnWave(false);
    spawnWave(true);
    return `${lastX},${lastY}`;
  };

  if (document.documentElement) install();
  else document.addEventListener("DOMContentLoaded", install, { once: true });
})();
