package uniffi.xcelerate;

/**
 * A running Chromium-family browser (Chrome, Chromium, Edge, Brave, …).
 */
public interface BrowserInterface {
    
    /**
     * Returns the plugin audit log as a JSON array (no secrets are recorded).
     */public java.lang.String auditLog();
    
    /**
     * Verifies the integrity of the append-only plugin audit log.
     */public boolean auditVerify();
    
    /**
     * Names of the plugins currently available on this browser.
     *
     * Xcelerate ships **no** built-in plugins, so this lists the plugins that
     * have been installed or loaded on this instance.
     */public java.util.List<java.lang.String> availablePlugins();
    
    /**
     * Returns the browser context ids as a JSON array.
     */public java.util.concurrent.CompletableFuture<java.lang.String> browserContexts() ;
    
    /**
     * Returns the browser version info as JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> capabilities() ;
    
    /**
     * Closes the browser, letting it flush the profile, then kills it if needed.
     *
     * An attached browser (from [`Browser::connect`]) is not owned by this
     * handle, so this is a no-op: it never sends `Browser.close` and never
     * kills the process.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> closeBrowser() ;
    
    /**
     * Returns all browser cookies as a JSON array.
     */public java.util.concurrent.CompletableFuture<java.lang.String> cookies() ;
    
    /**
     * Deletes cookies with the given name.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> deleteCookie(java.lang.String name) ;
    
    /**
     * Returns the registered event names.
     */public java.util.concurrent.CompletableFuture<java.util.List<java.lang.String>> eventNames();
    
    /**
     * Grants permissions (JSON array) to an origin.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> grantPermissions(java.lang.String origin, java.lang.String permissionsJson) ;
    
    /**
     * Whether the underlying connection is alive.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> isConnected();
    
    /**
     * Whether an event name is registered.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> listensTo(java.lang.String eventName);
    
    /**
     * Loads a plugin from disk.
     *
     * `path` may be a plugin directory (containing `plugin.json`) or a
     * `plugin.json` file. The manifest is validated, the `entrypoint` is
     * instantiated as a sandboxed WebAssembly component, and a `describe`
     * handshake wires up its ops. Dangerous capabilities stay denied unless
     * opted into via `XCELERATE_PLUGIN_ALLOW`.
     *
     * Once loaded, the plugin's ops are reachable through
     * `plugin(name).invoke(op, args_json)` in every language.
     */public java.lang.String loadPlugin(java.lang.String path) throws XcelerateException;
    
    /**
     * Creates a new (incognito) browser context and returns its id.
     */public java.util.concurrent.CompletableFuture<java.lang.String> newContext() ;
    public java.util.concurrent.CompletableFuture<Page> newPage(java.lang.String url) ;
    
    /**
     * Registers interest in a root-session CDP event.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> on(java.lang.String eventName);
    
    /**
     * Alias for [`Browser::on`].
     */public java.util.concurrent.CompletableFuture<java.lang.Void> once(java.lang.String eventName);
    
    /**
     * Returns a handle to an enabled plugin so its ops can be invoked.
     */public PluginHandle plugin(java.lang.String name) throws XcelerateException;
    
    /**
     * Names of the plugins currently enabled on this browser.
     */public java.util.List<java.lang.String> pluginNames();
    
    /**
     * Removes every registered listener.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> removeAllListeners();
    
    /**
     * Removes a single registered listener.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> removeListener(java.lang.String eventName);
    
    /**
     * Resets all permission overrides.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> resetPermissions() ;
    
    /**
     * Sets a cookie from a JSON object.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setCookie(java.lang.String cookieJson) ;
    
    /**
     * Sets the download directory for the browser.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setDownloadBehavior(java.lang.String path) ;
    
    /**
     * Starts CDP tracing.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> startTracing() ;
    
    /**
     * Stops CDP tracing.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> stopTracing() ;
    
    /**
     * Returns the current targets as a JSON array (`Target.getTargets`).
     */public java.util.concurrent.CompletableFuture<java.lang.String> targets() ;
    
    /**
     * Enables an installed plugin at runtime.
     *
     * Launch-time contributions (such as binary patching) only take effect if
     * the plugin was installed before the browser launched; enabling a plugin
     * afterwards applies its runtime hooks to pages created from now on. This
     * is audited as a runtime enable. Unknown names are refused.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> usePlugin(java.lang.String name) ;
    
    /**
     * Returns the browser's user agent.
     */public java.util.concurrent.CompletableFuture<java.lang.String> userAgent() ;
    
    /**
     * Returns the browser version information.
     */public java.util.concurrent.CompletableFuture<java.lang.String> version() ;
    
    /**
     * Waits for the next root-session CDP event named `event_name`.
     */public java.util.concurrent.CompletableFuture<java.lang.String> waitForEvent(java.lang.String eventName, long timeoutMs) ;
    
    /**
     * [`Browser::wait_for_event`] with the default 30s timeout.
     */public java.util.concurrent.CompletableFuture<java.lang.String> waitForEventDefault(java.lang.String eventName) ;
    
    /**
     * The WebSocket endpoint Chrome was launched with.
     */public java.lang.String wsEndpoint();
    
}

