(function() {
    const patchedFunctions = new Map();

    // Helper: Mask toString for modified functions to prevent "native code" detection
    const maskToString = (fn, name) => {
        patchedFunctions.set(fn, `function ${name}() { [native code] }`);
    };

    // Apply main Function.prototype.toString override
    const originalToString = Function.prototype.toString;
    Function.prototype.toString = function() {
        if (patchedFunctions.has(this)) {
            return patchedFunctions.get(this);
        }
        if (this === Function.prototype.toString) {
            return 'function toString() { [native code] }';
        }
        return originalToString.apply(this, arguments);
    };
    maskToString(Function.prototype.toString, 'toString');

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

    // 3. Mock window.chrome (Real Chrome always has this)
    if (!window.chrome) {
        const mockChrome = {
            app: {
                isInstalled: false,
                InstallState: { DISABLED: 'disabled', INSTALLED: 'installed', NOT_INSTALLED: 'not_installed' },
                RunningState: { CANNOT_RUN: 'cannot_run', READY_TO_RUN: 'ready_to_run', RUNNING: 'running' },
                getDetails: () => {},
                getIsInstalled: () => {},
                install: () => {}
            },
            runtime: {
                OnInstalledReason: { INSTALL: 'install', SHARED_MODULE_UPDATE: 'shared_module_update', UPDATE: 'update' },
                OnRestartRequiredReason: { APP_UPDATE: 'app_update', OS_UPDATE: 'os_update', PERIODIC: 'periodic' },
                PlatformArch: { ARM: 'arm', ARM64: 'arm64', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
                PlatformNaclArch: { ARM: 'arm', MIPS: 'mips', MIPS64: 'mips64', X86_32: 'x86-32', X86_64: 'x86-64' },
                PlatformOs: { ANDROID: 'android', CROS: 'cros', LINUX: 'linux', MAC: 'mac', OPENBSD: 'openbsd', WIN: 'win' },
                RequestUpdateCheckStatus: { NO_UPDATE: 'no_update', THROTTLED: 'throttled', UPDATE_AVAILABLE: 'update_available' }
            },
            loadTimes: () => {},
            csi: () => {}
        };
        
        maskToString(mockChrome.app.getDetails, 'getDetails');
        maskToString(mockChrome.app.getIsInstalled, 'getIsInstalled');
        maskToString(mockChrome.app.install, 'install');
        maskToString(mockChrome.loadTimes, 'loadTimes');
        maskToString(mockChrome.csi, 'csi');

        Object.defineProperty(window, 'chrome', {
            get: () => mockChrome,
            configurable: true,
            enumerable: true
        });
    }

    // 4. Mask permissions check (notifications should match normal permission state)
    if (window.navigator && window.navigator.permissions) {
        const originalQuery = window.navigator.permissions.query;
        window.navigator.permissions.query = (parameters) => (
            parameters.name === 'notifications' ?
                Promise.resolve({ state: Notification.permission }) :
                originalQuery(parameters)
        );
        maskToString(window.navigator.permissions.query, 'query');
    }

    // 5. Mock plugins (Essential for consistency in Headless)
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
                
                maskToString(p.refresh, 'refresh');
                maskToString(p.item, 'item');
                maskToString(p.namedItem, 'namedItem');
                return p;
            },
            configurable: true,
            enumerable: true
        });
    }

    // 6. Mock languages (Ensure they are populated with en-US/en)
    if (!navigator.languages || navigator.languages.length === 0) {
        Object.defineProperty(navigator, 'languages', {
            get: () => ['en-US', 'en'],
            configurable: true,
            enumerable: true
        });
    }

    // 7. Mock WebGL Vendor & Renderer (Bypass GPU/Hardware Checks)
    const spoofWebGL = (ctx) => {
        if (!ctx) return;
        const originalGetParameter = ctx.prototype.getParameter;
        ctx.prototype.getParameter = function(parameter) {
            // UNMASKED_VENDOR_WEBGL
            if (parameter === 37445) {
                return 'Google Inc. (NVIDIA)';
            }
            // UNMASKED_RENDERER_WEBGL
            if (parameter === 37446) {
                return 'ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)';
            }
            return originalGetParameter.apply(this, arguments);
        };
        maskToString(ctx.prototype.getParameter, 'getParameter');
    };
    if (window.WebGLRenderingContext) spoofWebGL(WebGLRenderingContext);
    if (window.WebGL2RenderingContext) spoofWebGL(WebGL2RenderingContext);

    // 8. Mock Device Hardware metrics if they default to headless values
    if (!navigator.deviceMemory || navigator.deviceMemory < 4) {
        Object.defineProperty(navigator, 'deviceMemory', {
            get: () => 8,
            configurable: true,
            enumerable: true
        });
    }
    if (!navigator.hardwareConcurrency || navigator.hardwareConcurrency < 2) {
        Object.defineProperty(navigator, 'hardwareConcurrency', {
            get: () => 8,
            configurable: true,
            enumerable: true
        });
    }

})();
