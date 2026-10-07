/* xcelerate guardian - neutralise the usual ways a page disrupts automation.
 *
 * Install it so it runs BEFORE any page script on every document:
 *
 *   Rust:  page.inject_file("guardian.js".into()).await?;         // every navigation
 *          page.add_script_to_evaluate_on_new_document(src).await?;
 *   CLI:   xcelerate ... session
 *          xcelerate> guard .agents/skills/hostile-pages/guardian.js
 *
 * It is idempotent and defensive: every step is wrapped so a page that
 * redefines or removes a primitive cannot break the script.
 */
(() => {
  if (window.__xcelerateGuard) return;
  window.__xcelerateGuard = true;

  // --- 1. Popups and new tabs ------------------------------------------------
  // Method A: neuter window.open. Sites open "ads" and redirect traps this way.
  try {
    window.open = function () {
      console.warn("[xcelerate] blocked window.open ->", arguments[0] || "");
      return null;
    };
  } catch (_) {}

  // Method B: strip target on links so click-throughs stay in this tab. An
  // unbounded `target="_blank"` is how a stray click spawns a second window.
  document.addEventListener(
    "click",
    (event) => {
      const el = event.target;
      const anchor = el && el.closest ? el.closest("a[target]") : null;
      if (anchor) anchor.removeAttribute("target");
    },
    true,
  );

  // --- 2. Navigation traps ---------------------------------------------------
  // beforeunload prompts ("are you sure you want to leave?") freeze teardown.
  try {
    window.onbeforeunload = null;
    window.addEventListener(
      "beforeunload",
      (e) => e.stopImmediatePropagation(),
      true,
    );
  } catch (_) {}

  // --- 3. Autoplay media -----------------------------------------------------
  // A 5s ad video that hijacks the video element looks like "the page" to a
  // naive reader. Mute anything that starts playing.
  document.addEventListener(
    "play",
    (e) => {
      try {
        if (e.target && "muted" in e.target) e.target.muted = true;
      } catch (_) {}
    },
    true,
  );

  // --- 4. Ad frames and containers ------------------------------------------
  const AD_HOST =
    /(doubleclick|googlesyndication|googleadservices|adservice|adsystem|amazon-adsystem|taboola|outbrain|pubmatic|rubiconproject|criteo|adnxs|moatads|bidswitch|casalemedia|33across|sharethrough|yieldmo|adnxs|adroll)/i;
  const AD_HINT =
    /(^|[-_\s])(ad|ads|advert|advertise|advertisement|sponsor|sponsored|promo|promoted|adslot|adunit|google_ads|adsbygoogle)([-_\s]|$)/i;

  const removeAds = () => {
    const pool = document.querySelectorAll(
      'iframe[src], ins.adsbygoogle, [id^="google_ads_"], [data-ad], [class*="adsbygoogle"], [aria-label*="advert" i]',
    );
    pool.forEach((el) => {
      const hay =
        (el.getAttribute("src") || "") +
        " " +
        (el.id || "") +
        " " +
        (el.className || "");
      if (AD_HOST.test(hay) || AD_HINT.test(hay)) el.remove();
    });
  };

  // --- 5. Ad interstitials ---------------------------------------------------
  // Auto-click a close control ONLY when its accessible name ties it to an ad,
  // so a legitimate app modal ("got it", "continue") is left for the agent.
  const closeAdModals = () => {
    const controls = document.querySelectorAll('button, [role="button"], a');
    controls.forEach((el) => {
      const label = ((el.innerText || "") + " " + (el.getAttribute("aria-label") || "")).trim();
      if (/(close|dismiss|skip|hide|remove).*(ad|advert|sponsor)/i.test(label)) {
        try {
          el.click();
        } catch (_) {}
      }
    });
  };

  const sweep = () => {
    try {
      removeAds();
      closeAdModals();
    } catch (_) {}
  };

  let scheduled = false;
  const schedule = () => {
    if (scheduled) return;
    scheduled = true;
    const run = () => {
      scheduled = false;
      sweep();
    };
    if (typeof requestAnimationFrame === "function") requestAnimationFrame(run);
    else setTimeout(run, 16);
  };

  const start = () => {
    sweep();
    try {
      new MutationObserver(schedule).observe(document.documentElement, {
        childList: true,
        subtree: true,
      });
    } catch (_) {}
  };
  if (document.documentElement) start();
  else document.addEventListener("DOMContentLoaded", start, { once: true });

  // --- 6. Agent escape hatch -------------------------------------------------
  // window.__xcelerateDismiss(/regex/i) -> number of controls clicked.
  window.__xcelerateDismiss = (re) => {
    let clicked = 0;
    document
      .querySelectorAll('button, [role="button"], a, [role="link"]')
      .forEach((el) => {
        const label = ((el.innerText || "") + " " + (el.getAttribute("aria-label") || "")).trim();
        if (re.test(label)) {
          try {
            el.click();
            clicked++;
          } catch (_) {}
        }
      });
    return clicked;
  };
})();
