package uniffi.xcelerate;

public interface PageInterface {
    
    /**
     * Activates this page's target.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> activate() ;
    
    /**
     * Activates the given target (window/tab).
     */public java.util.concurrent.CompletableFuture<java.lang.Void> activateTarget(java.lang.String targetId) ;
    
    /**
     * Evaluates a script on every new document.
     */public java.util.concurrent.CompletableFuture<java.lang.String> addScriptToEvaluateOnNewDocument(java.lang.String source) ;
    
    /**
     * Injects a `<style>` element and returns the injected content.
     */public java.util.concurrent.CompletableFuture<java.lang.String> addStyleTag(java.lang.String content) ;
    
    /**
     * Sets the credentials used to answer HTTP auth challenges.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> authenticate(java.lang.String username, java.lang.String password) ;
    
    /**
     * Brings the page to the front.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> bringToFront() ;
    
    /**
     * Like [`Page::call_json`] but coerces the result to a bool.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> callBool(java.lang.String function, java.lang.String argsJson) ;
    
    /**
     * Calls a JS function with JSON-encoded arguments, returning JSON text.
     *
     * This is the shim the adapters use to express the broad upstream surface
     * as data (a function body per method) rather than a core method per member.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callJson(java.lang.String function, java.lang.String argsJson) ;
    
    /**
     * Runs a JS function against the element matching `selector` (`$eval`).
     */public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelector(java.lang.String selector, java.lang.String expression) ;
    
    /**
     * Runs a JS function against every element matching `selector` (`$$eval`).
     */public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelectorAll(java.lang.String selector, java.lang.String expression) ;
    
    /**
     * Like [`Page::call_json`] but coerces the result to a string.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callString(java.lang.String function, java.lang.String argsJson) ;
    
    /**
     * Clears the recorded intercepted requests.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> clearRequests();
    
    /**
     * Moves the mouse to (x, y) and performs a click (down & up) with human-like delays.
     */public java.util.concurrent.CompletableFuture<Page> clickMouse(double x, double y) ;
    
    /**
     * Closes the page.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> closePage() ;
    
    /**
     * Returns the full HTML content of the page.
     */public java.util.concurrent.CompletableFuture<java.lang.String> content() ;
    
    /**
     * Returns a single cookie by name as JSON (or null).
     */public java.util.concurrent.CompletableFuture<java.lang.String> cookie(java.lang.String name) ;
    
    /**
     * Returns the cookies visible to this page as a JSON array.
     */public java.util.concurrent.CompletableFuture<java.lang.String> cookies() ;
    
    /**
     * Starts CSS coverage collection.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> coverageStartCss() ;
    
    /**
     * Starts JS coverage collection.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> coverageStartJs() ;
    
    /**
     * Stops CSS coverage collection and returns the result as JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> coverageStopCss() ;
    
    /**
     * Stops JS coverage collection and returns the result as JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> coverageStopJs() ;
    
    /**
     * Returns the page PDF as a base64 string.
     */public java.util.concurrent.CompletableFuture<java.lang.String> createPdfStream() ;
    public byte[] decodeBase64(java.lang.String data) throws XcelerateException;
    
    /**
     * Overrides the idle state.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> emulateIdleState(boolean isUserActive, boolean isScreenUnlocked) ;
    
    /**
     * Emulates a media type and/or colour scheme.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> emulateMedia(java.lang.String media, java.lang.String colorScheme) ;
    public java.util.concurrent.CompletableFuture<java.lang.Void> ensureInterception();
    
    /**
     * Evaluates JavaScript and coerces the result to a bool.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> evaluateBool(java.lang.String expression) ;
    
    /**
     * Evaluates JavaScript and returns the resulting object as an [`Element`].
     */public java.util.concurrent.CompletableFuture<Element> evaluateHandle(java.lang.String expression) ;
    
    /**
     * Evaluates JavaScript in the page and returns the result as a JSON string.
     */public java.util.concurrent.CompletableFuture<java.lang.String> evaluateJson(java.lang.String expression) ;
    
    /**
     * Evaluates JavaScript and coerces the result to a string.
     */public java.util.concurrent.CompletableFuture<java.lang.String> evaluateString(java.lang.String expression) ;
    
    /**
     * Returns the registered event names.
     */public java.util.concurrent.CompletableFuture<java.util.List<java.lang.String>> eventNames();
    
    /**
     * Escape hatch: sends an arbitrary CDP command and returns its JSON result.
     *
     * The adapters use this to express CDP-backed library methods as data
     * (a method name + parameter template), keeping the core surface small.
     */public java.util.concurrent.CompletableFuture<java.lang.String> executeCdpCmd(java.lang.String method, java.lang.String paramsJson) ;
    
    /**
     * Finds an element matching the CSS selector.
     */public java.util.concurrent.CompletableFuture<Element> findElement(java.lang.String selector) ;
    
    /**
     * Returns the frame matching an id or name as JSON (or null).
     */public java.util.concurrent.CompletableFuture<java.lang.String> frame(java.lang.String frameId) ;
    
    /**
     * Returns the main frame's name.
     */public java.util.concurrent.CompletableFuture<java.lang.String> frameName() ;
    
    /**
     * Returns every frame in the page as a JSON array.
     */public java.util.concurrent.CompletableFuture<java.lang.String> frames() ;
    
    /**
     * Finds a form control by its associated `<label>` text.
     */public java.util.concurrent.CompletableFuture<Element> getByLabel(java.lang.String label) ;
    
    /**
     * Finds an element by ARIA role (falls back to a tag-name lookup).
     */public java.util.concurrent.CompletableFuture<Element> getByRole(java.lang.String role) ;
    
    /**
     * Finds an element whose text content contains `text`.
     */public java.util.concurrent.CompletableFuture<Element> getByText(java.lang.String text) ;
    
    /**
     * Returns the stored default timeout (ms).
     */public java.util.concurrent.CompletableFuture<java.lang.Double> getDefaultTimeout() ;
    public java.util.concurrent.CompletableFuture<java.lang.Void> goBack() ;
    
    /**
     * Navigates forward in history.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> goForward() ;
    
    /**
     * Accepts or dismisses the active JavaScript dialog.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> handleJsDialog(boolean accept, java.lang.String promptText) ;
    
    /**
     * Reads a local file and injects it as an init script.
     */public java.util.concurrent.CompletableFuture<java.lang.String> injectFile(java.lang.String path) ;
    
    /**
     * Whether drag interception is enabled.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> isDragInterceptionEnabled();
    
    /**
     * Dispatches a keydown event for `key`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardDown(java.lang.String key) ;
    
    /**
     * Presses `key` on the focused element.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardPress(java.lang.String key) ;
    
    /**
     * Types `text` into the focused element.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardType(java.lang.String text) ;
    
    /**
     * Dispatches a keyup event for `key`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardUp(java.lang.String key) ;
    
    /**
     * Whether an event name is registered.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> listensTo(java.lang.String eventName);
    
    /**
     * Returns the main frame as JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> mainFrame() ;
    
    /**
     * Returns the page performance metrics as a JSON object.
     */public java.util.concurrent.CompletableFuture<java.lang.String> metrics() ;
    
    /**
     * Triggers a mousePress event at the current mouse coordinates.
     */public java.util.concurrent.CompletableFuture<Page> mouseDown(java.lang.String button) ;
    
    /**
     * Triggers a mouseReleased event at the current mouse coordinates.
     */public java.util.concurrent.CompletableFuture<Page> mouseUp(java.lang.String button) ;
    
    /**
     * Moves the mouse cursor from the current position to the target (x, y) along a realistic Bezier curve.
     */public java.util.concurrent.CompletableFuture<Page> moveMouse(double x, double y) ;
    
    /**
     * Navigates to a URL.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> navigate(java.lang.String url) ;
    
    /**
     * Registers interest in a CDP event name.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> on(java.lang.String eventName);
    
    /**
     * Alias for [`Page::on`].
     */public java.util.concurrent.CompletableFuture<java.lang.Void> once(java.lang.String eventName);
    public java.util.concurrent.CompletableFuture<byte[]> pdf() ;
    
    /**
     * Focuses the element matching `selector` and presses `key`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> press(java.lang.String selector, java.lang.String key) ;
    
    /**
     * Returns every element matching the CSS selector.
     *
     * Uses two round trips (fetch the node list, then read its properties)
     * instead of one `evaluate` per match.
     */public java.util.concurrent.CompletableFuture<java.util.List<Element>> querySelectorAll(java.lang.String selector) ;
    
    /**
     * Returns the first node matching an XPath expression as an [`Element`].
     */public java.util.concurrent.CompletableFuture<Element> querySelectorXpath(java.lang.String xpath) ;
    public java.util.concurrent.CompletableFuture<java.lang.String> rawWindowBounds() ;
    
    /**
     * Reloads the page.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> reload() ;
    
    /**
     * Removes every registered event listener.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> removeAllListeners();
    
    /**
     * Removes a single registered event listener.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> removeListener(java.lang.String eventName);
    
    /**
     * Removes an init script by identifier.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> removeScript(java.lang.String identifier) ;
    
    /**
     * Returns the most recent intercepted request as JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> request() ;
    
    /**
     * Returns the intercepted requests seen so far as JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> requests() ;
    
    /**
     * Adds a route rule. `action` is `continue`, `abort`, or `fulfill`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> route(java.lang.String pattern, java.lang.String action, java.lang.String body, java.lang.String contentType) ;
    
    /**
     * Aborts every request matching `pattern`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> routeAbort(java.lang.String pattern) ;
    
    /**
     * Registers fulfill routes for every entry in a HAR file.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> routeFromHar(java.lang.String path) ;
    
    /**
     * Fulfills every request matching `pattern` with `body`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> routeFulfill(java.lang.String pattern, java.lang.String body, java.lang.String contentType) ;
    public java.util.concurrent.CompletableFuture<byte[]> screenshot() ;
    public java.util.concurrent.CompletableFuture<byte[]> screenshotFull() ;
    
    /**
     * Selects options by value/label on the matching `<select>`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> selectOption(java.lang.String selector, java.lang.String valuesJson) ;
    
    /**
     * Enables or disables the HTTP cache.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setCacheEnabled(boolean enabled) ;
    
    /**
     * Replaces the document content.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setContent(java.lang.String html) ;
    
    /**
     * Stores a default timeout (ms) for adapter compatibility.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setDefaultTimeout(double milliseconds) ;
    
    /**
     * Enables or disables input drag interception.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setDragInterception(boolean enabled) ;
    
    /**
     * Overrides media features (JSON array of `{name,value}`).
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setEmulatedMediaFeatures(java.lang.String featuresJson) ;
    
    /**
     * Sets extra HTTP headers for every request from this page.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setExtraHttpHeaders(java.lang.String headersJson) ;
    
    /**
     * Sets the files of the matching `<input type="file">`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setInputFiles(java.lang.String selector, java.lang.String filesJson) ;
    
    /**
     * Enables or disables JavaScript execution.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setJavascriptEnabled(boolean enabled) ;
    
    /**
     * Toggles offline mode.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setOffline(boolean offline) ;
    
    /**
     * Enables or disables request interception for this page.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setRequestInterception(boolean enabled) ;
    
    /**
     * Restores cookies + localStorage from a storage-state JSON object.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setStorageState(java.lang.String stateJson) ;
    
    /**
     * Overrides `navigator.userAgent` for this page.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setUserAgent(java.lang.String userAgent, java.lang.String acceptLanguage) ;
    
    /**
     * Overrides the viewport size.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setViewportSize(long width, long height) ;
    
    /**
     * Moves and resizes the window.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowBounds(long left, long top, long width, long height) ;
    
    /**
     * Moves the window, preserving its size.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowPosition(long x, long y) ;
    
    /**
     * Resizes the window, preserving its position.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowSize(long width, long height) ;
    
    /**
     * Sets the window state (`normal` | `minimized` | `maximized` | `fullscreen`).
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowState(java.lang.String state) ;
    
    /**
     * Starts a PNG screencast.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> startScreencast() ;
    
    /**
     * Starts CDP tracing on this page's session.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> startTracing() ;
    
    /**
     * Stops the screencast.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> stopScreencast() ;
    
    /**
     * Stops CDP tracing on this page's session.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> stopTracing() ;
    
    /**
     * Returns cookies + localStorage as a storage-state JSON object.
     */public java.util.concurrent.CompletableFuture<java.lang.String> storageState() ;
    
    /**
     * The CDP target id backing this page.
     */public java.lang.String targetId();
    
    /**
     * Returns the page title.
     */public java.util.concurrent.CompletableFuture<java.lang.String> title() ;
    
    /**
     * Dispatches a touch tap at (x, y).
     */public java.util.concurrent.CompletableFuture<java.lang.Void> touchTap(double x, double y) ;
    
    /**
     * Removes the routes registered for `pattern`.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> unroute(java.lang.String pattern) ;
    
    /**
     * Removes every route rule.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> unrouteAll() ;
    
    /**
     * Returns the current document URL.
     */public java.util.concurrent.CompletableFuture<java.lang.String> url() ;
    
    /**
     * Waits for the next CDP event named `event_name` and returns its params.
     *
     * The relevant domain is enabled first (best effort), so callers do not
     * have to.
     */public java.util.concurrent.CompletableFuture<java.lang.String> waitForEvent(java.lang.String eventName, long timeoutMs) ;
    
    /**
     * [`Page::wait_for_event`] with the default 30s timeout.
     */public java.util.concurrent.CompletableFuture<java.lang.String> waitForEventDefault(java.lang.String eventName) ;
    
    /**
     * Polls `expression` until it evaluates truthy or `timeout_ms` elapses.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> waitForFunction(java.lang.String expression, long timeoutMs) ;
    
    /**
     * Waits for the page to finish loading.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> waitForNavigation() ;
    
    /**
     * Waits for an element matching the selector to appear in the DOM.
     */public java.util.concurrent.CompletableFuture<Element> waitForSelector(java.lang.String selector) ;
    
    /**
     * Waits for the first XPath match to appear.
     */public java.util.concurrent.CompletableFuture<Element> waitForXpath(java.lang.String xpath, long timeoutMs) ;
    public java.util.concurrent.CompletableFuture<java.lang.Long> windowId() ;
    
    /**
     * Returns the window position as JSON (`{x,y}`).
     */public java.util.concurrent.CompletableFuture<java.lang.String> windowPosition() ;
    
    /**
     * Returns the window bounds as JSON (`{left,top,width,height,windowState}`).
     */public java.util.concurrent.CompletableFuture<java.lang.String> windowRect() ;
    
    /**
     * Returns the window size as JSON (`{width,height}`).
     */public java.util.concurrent.CompletableFuture<java.lang.String> windowSize() ;
    
}

