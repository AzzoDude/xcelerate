(function() {
    // 1. Completely remove navigator.webdriver
    try {
        const newProto = Object.getPrototypeOf(navigator);
        delete newProto.webdriver;
    } catch(e) {}
    try {
        delete navigator.webdriver;
    } catch(e) {}

    // 2. Hide Chrome signals (CDC and others)
    const hideLeaks = (obj) => {
        try {
            const props = Object.getOwnPropertyNames(obj);
            for (const prop of props) {
                if (prop.includes('cdc_') || prop.includes('__$cdc_')) {
                    try { delete obj[prop]; } catch(e) {}
                }
            }
        } catch(e) {}
    };
    hideLeaks(window);
    hideLeaks(document);
    hideLeaks(Navigator.prototype);

    // 3. Mask permissions check
    if (window.navigator && window.navigator.permissions) {
        const originalQuery = window.navigator.permissions.query;
        window.navigator.permissions.query = (parameters) => (
            parameters.name === 'notifications' ?
                Promise.resolve({ state: Notification.permission }) :
                originalQuery(parameters)
        );
    }

    // 4. Mock plugins (Essential for consistency in Headless)
    if (navigator.plugins.length === 0) {
        const mockPlugins = [
            { name: 'PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
            { name: 'Chrome PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
            { name: 'Chromium PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
            { name: 'Microsoft Edge PDF Viewer', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
            { name: 'WebKit built-in PDF', filename: 'internal-pdf-viewer', description: 'Portable Document Format' }
        ];

        Object.defineProperty(navigator, 'plugins', {
            get: () => {
                const p = [...mockPlugins];
                p.refresh = () => {};
                p.item = (i) => p[i];
                p.namedItem = (n) => p.find(x => x.name === n);
                return p;
            },
            configurable: true,
            enumerable: true
        });
    }

    // 5. Hide WebRTC (Often leaks the fact that it's a controlled browser)
    // We don't disable it, just mask some properties if needed.

    // 6. Mask toString for all modified functions to prevent "native code" detection
    const originalToString = Function.prototype.toString;
    const patchedFunctions = new Map();
    
    if (window.navigator && window.navigator.permissions) {
        patchedFunctions.set(window.navigator.permissions.query, 'function query() { [native code] }');
    }

    Function.prototype.toString = function() {
        if (patchedFunctions.has(this)) {
            return patchedFunctions.get(this);
        }
        if (this === Function.prototype.toString) {
            return 'function toString() { [native code] }';
        }
        return originalToString.apply(this, arguments);
    };

})();
