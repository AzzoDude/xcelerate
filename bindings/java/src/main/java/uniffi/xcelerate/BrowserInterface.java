package uniffi.xcelerate;

/**
 * Represents a browser instance (e.g., Chrome or Edge).
 */
public interface BrowserInterface {
    
    /**
     * Returns the browser context ids as a JSON array.
     */public java.util.concurrent.CompletableFuture<java.lang.String> browserContexts() ;
    
    /**
     * Returns the browser version info as JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> capabilities() ;
    
    /**
     * Closes the browser and kills the process.
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

