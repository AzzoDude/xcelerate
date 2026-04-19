(function() {
    // 1. Remove the webdriver property (the basic check)
    Object.defineProperty(navigator, 'webdriver', {get: () => false});

    // 2. Scan and delete CDC leaks
    const keys = Object.keys(window);
    for (let i = 0; i < keys.length; i++) {
        if (keys[i].includes('cdc_')) {
            delete window[keys[i]];
        }
    }

    // 3. Mask the "CDC" Proxy behavior
    // Some detectors check if standard functions look like Proxies.
    const originalQuery = document.querySelector;
    document.querySelector = function() {
        return originalQuery.apply(this, arguments);
    };
    document.querySelector.toString = () => "function querySelector() { [native code] }";
})();