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
     */public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelector(java.lang.String selector, java.lang.String expression) ;
    
    /**
     * Runs a JS function against every descendant matching `selector`.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelectorAll(java.lang.String selector, java.lang.String expression) ;
    
    /**
     * Like [`Element::call_json`] but coerces the result to a string.
     */public java.util.concurrent.CompletableFuture<java.lang.String> callString(java.lang.String function, java.lang.String argsJson) ;
    
    /**
     * Clicks the element.
     */public java.util.concurrent.CompletableFuture<Element> click() ;
    
    /**
     * Clicks the element using realistic mouse movement and CDP input events.
     */public java.util.concurrent.CompletableFuture<Element> clickStealth() ;
    
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
     */public java.util.concurrent.CompletableFuture<Element> getByLabel(java.lang.String label) ;
    
    /**
     * Finds a descendant by ARIA role.
     */public java.util.concurrent.CompletableFuture<Element> getByRole(java.lang.String role) ;
    
    /**
     * Finds a descendant whose text contains `text`.
     */public java.util.concurrent.CompletableFuture<Element> getByText(java.lang.String text) ;
    
    /**
     * Returns this element's enumerable properties as a JSON object.
     */public java.util.concurrent.CompletableFuture<java.lang.String> getProperties() ;
    
    /**
     * Hovers over the element.
     */public java.util.concurrent.CompletableFuture<Element> hover() ;
    
    /**
     * Hovers over the element using realistic mouse movement.
     */public java.util.concurrent.CompletableFuture<Element> hoverStealth() ;
    
    /**
     * Returns the inner HTML of the element.
     */public java.util.concurrent.CompletableFuture<java.lang.String> innerHtml() ;
    
    /**
     * Focuses the element and presses a key.
     */public java.util.concurrent.CompletableFuture<java.lang.Void> press(java.lang.String key) ;
    
    /**
     * Returns the first descendant matching `selector` as an [`Element`].
     */public java.util.concurrent.CompletableFuture<Element> querySelector(java.lang.String selector) ;
    
    /**
     * Returns every descendant matching `selector`.
     *
     * Resolves the whole node list with a single `Runtime.getProperties` call
     * rather than one `evaluate` per match.
     */public java.util.concurrent.CompletableFuture<java.util.List<Element>> querySelectorAll(java.lang.String selector) ;
    
    /**
     * Finds a descendant by attribute value.
     */public java.util.concurrent.CompletableFuture<Element> querySelectorAttr(java.lang.String attribute, java.lang.String value) ;
    
    /**
     * Finds a descendant matching an XPath expression.
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
     */public java.util.concurrent.CompletableFuture<Element> waitForSelector(java.lang.String selector) ;
    
}

