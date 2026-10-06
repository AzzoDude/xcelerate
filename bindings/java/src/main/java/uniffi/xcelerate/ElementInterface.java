package uniffi.xcelerate;

/**
 * Represents an HTML element in the DOM.
 */
public interface ElementInterface {
    
    /**
     * Returns the value of a specific attribute.
     */public java.util.concurrent.CompletableFuture<java.lang.String> attribute(java.lang.String name) ;
    
    /**
     * Like [`Element::call_json`] but coerces the result to a bool.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> callBool(java.lang.String function, java.lang.String argsJson) ;
    
    /**
     * Calls a JS function on this element with JSON-encoded arguments.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callJson(java.lang.String function, java.lang.String argsJson) ;
    
    /**
     * Runs a JS function against the first descendant matching `selector`.
     *
     * The descendant is resolved with the shadow-piercing selector first, so
     * the expression also runs against a match inside an open shadow root.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelector(java.lang.String selector, java.lang.String expression) ;
    
    /**
     * Runs a JS function against every descendant matching `selector`.
     *
     * The descendants are resolved with the shadow-piercing selector first, so
     * matches inside open shadow roots are included too.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelectorAll(java.lang.String selector, java.lang.String expression) ;
    
    /**
     * Like [`Element::call_json`] but coerces the result to a string.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callString(java.lang.String function, java.lang.String argsJson) ;
    
    /**
     * Clicks the element.
     */public java.util.concurrent.CompletableFuture<Element> click() ;
    
    /**
     * Clicks the element using realistic mouse movement and CDP input events.
     *
     * Fails with [`XcelerateError::NotFound`] if the element is not actionable
     * (zero-size, `display:none`, `visibility:hidden` or fully transparent),
     * rather than dispatching a click at coordinates that nothing occupies.
     */public java.util.concurrent.CompletableFuture<Element> clickMouse() ;
    
    /**
     * Number of elements this handle represents (always 1).
     */public java.util.concurrent.CompletableFuture<java.lang.Long> count() ;
    
    /**
     * Releases the underlying remote object handle.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> dispose() ;
    
    /**
     * Calls a function on this element and coerces the result to a bool.
     */public java.util.concurrent.CompletableFuture<java.lang.Boolean> evaluateBool(java.lang.String function) ;
    
    /**
     * Calls a JS function on this element and returns the resulting node.
     */public java.util.concurrent.CompletableFuture<Element> evaluateHandle(java.lang.String function) ;
    
    /**
     * Calls a function on this element; the result is returned as JSON text.
     */public java.util.concurrent.CompletableFuture<java.lang.String> evaluateJson(java.lang.String function) ;
    
    /**
     * Calls a function on this element and coerces the result to a string.
     */public java.util.concurrent.CompletableFuture<java.lang.String> evaluateString(java.lang.String function) ;
    
    /**
     * Focuses the element.
     */public java.util.concurrent.CompletableFuture<Element> focus() ;
    
    /**
     * Finds a descendant form control by its `<label>` text.
     *
     * The `<label>` search pierces open shadow roots, and the associated control
     * is resolved from the label's own root so shadow-encapsulated controls work.
     */public java.util.concurrent.CompletableFuture<Element> getByLabel(java.lang.String label) ;
    
    /**
     * Finds a descendant by ARIA role.
     *
     * Prefers an explicit `[role="..."]` match, then falls back to the role name
     * as a tag, since a native `<button>`/`<a>` carries its role implicitly.
     * Both searches pierce open shadow roots.
     */public java.util.concurrent.CompletableFuture<Element> getByRole(java.lang.String role) ;
    
    /**
     * Finds a descendant whose text contains `text`.
     *
     * The search pierces open shadow roots.
     */public java.util.concurrent.CompletableFuture<Element> getByText(java.lang.String text) ;
    
    /**
     * Returns this element's enumerable properties as a JSON object.
     */public java.util.concurrent.CompletableFuture<java.lang.String> getProperties() ;
    
    /**
     * Hovers over the element.
     */public java.util.concurrent.CompletableFuture<Element> hover() ;
    
    /**
     * Hovers over the element using realistic mouse movement.
     */public java.util.concurrent.CompletableFuture<Element> hoverMouse() ;
    
    /**
     * Returns the inner HTML of the element.
     */public java.util.concurrent.CompletableFuture<java.lang.String> innerHtml() ;
    
    /**
     * Focuses the element and presses a key.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> press(java.lang.String key) ;
    
    /**
     * Returns the first descendant matching `selector` as an [`Element`].
     *
     * The search pierces open shadow roots, so web components are reachable.
     */public java.util.concurrent.CompletableFuture<Element> querySelector(java.lang.String selector) ;
    
    /**
     * Returns every descendant matching `selector`.
     *
     * The search pierces open shadow roots. Resolves the whole node list with a
     * single `Runtime.getProperties` call rather than one `evaluate` per match.
     */public java.util.concurrent.CompletableFuture<java.util.List<Element>> querySelectorAll(java.lang.String selector) ;
    
    /**
     * Finds a descendant by attribute value.
     */public java.util.concurrent.CompletableFuture<Element> querySelectorAttr(java.lang.String attribute, java.lang.String value) ;
    
    /**
     * Finds a descendant matching an XPath expression.
     *
     * The expression is evaluated over the composed tree - open shadow roots and
     * same-origin iframe documents are searched - by a built-in subset evaluator.
     * Expressions outside that subset (unions, extra axes, `count()`, ...) fall
     * back to the browser's native `document.evaluate`, which handles the full
     * language but does not pierce shadow roots.
     */public java.util.concurrent.CompletableFuture<Element> querySelectorXpath(java.lang.String xpath) ;
    
    /**
     * Captures a PNG screenshot cropped to this element.
     */public java.util.concurrent.CompletableFuture<byte[]> screenshot() ;
    
    /**
     * Captures a base64 PNG screenshot cropped to this element.
     */public java.util.concurrent.CompletableFuture<java.lang.String> screenshotBase64() ;
    
    /**
     * Selects options by value or label on this `<select>` element.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> selectOption(java.lang.String valuesJson) ;
    
    /**
     * Sets the files of this `<input type="file">` element.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> setInputFiles(java.lang.String filesJson) ;
    
    /**
     * Returns the visible text of the element.
     */public java.util.concurrent.CompletableFuture<java.lang.String> text() ;
    public java.util.concurrent.CompletableFuture<Element> typeText(java.lang.String text) ;
    
    /**
     * Waits for a descendant matching `selector` to appear.
     *
     * The wait happens inside the page in a single CDP call: a `MutationObserver`
     * resolves as soon as the node appears (and a slow rescan covers shadow
     * roots), instead of the caller polling `query_selector` over the wire every
     * 250ms. Times out after 30 seconds.
     */public java.util.concurrent.CompletableFuture<Element> waitForSelector(java.lang.String selector) ;
    
}

