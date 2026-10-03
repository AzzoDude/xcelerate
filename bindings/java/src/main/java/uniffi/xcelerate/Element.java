package uniffi.xcelerate;

/**
 * Represents an HTML element in the DOM.
 */
public class Element implements AutoCloseable, ElementInterface {
  protected long handle;
  protected UniffiCleaner.Cleanable cleanable;

  private java.util.concurrent.atomic.AtomicBoolean wasDestroyed = new java.util.concurrent.atomic.AtomicBoolean(false);
  private java.util.concurrent.atomic.AtomicLong callCounter = new java.util.concurrent.atomic.AtomicLong(1);

  /**
   * Internal constructor to wrap a raw handle from FFI.
   * The UniffiWithHandle marker disambiguates this from other constructors.
   */
  public Element(UniffiWithHandle phantom, long handle) {
    this.handle = handle;
    this.cleanable = UniffiLib.CLEANER.register(this, new UniffiCleanAction(handle));
  }

  /**
   * This constructor can be used to instantiate a fake object. Only used for tests. Any
   * attempt to actually use an object constructed this way will fail as there is no
   * connected Rust object.
   */
  public Element(NoHandle noHandle) {
    this.handle = 0L;
    this.cleanable = null;
  }

  

  @Override
  public synchronized void close() {
    // Only allow a single call to this method.
    // TODO(uniffi): maybe we should log a warning if called more than once?
    if (this.wasDestroyed.compareAndSet(false, true)) {
      // This decrement always matches the initial count of 1 given at creation time.
      if (this.callCounter.decrementAndGet() == 0L) {
        if (cleanable != null) {
          cleanable.clean();
        }
      }
    }
  }

  public <R> R callWithHandle(java.util.function.Function<java.lang.Long, R> block) {
    // Check and increment the call counter, to keep the object alive.
    // This needs a compare-and-set retry loop in case of concurrent updates.
    long c;
    do {
      c = this.callCounter.get();
      if (c == 0L) {
        throw new java.lang.IllegalStateException("Element object has already been destroyed");
      }
      if (c == java.lang.Long.MAX_VALUE) {
        throw new java.lang.IllegalStateException("Element call counter would overflow");
      }
    } while (! this.callCounter.compareAndSet(c, c + 1L));
    // Now we can safely do the method call without the handle being freed concurrently.
    try {
      return block.apply(this.uniffiCloneHandle());
    } finally {
      // This decrement always matches the increment we performed above.
      if (this.callCounter.decrementAndGet() == 0L) {
        if (cleanable != null) {
          cleanable.clean();
        }
      }
    }
  }

  public void callWithHandle(java.util.function.Consumer<java.lang.Long> block) {
    callWithHandle((java.lang.Long uniffiHandle) -> {
      block.accept(uniffiHandle);
      return (java.lang.Void)null;
    });
  }

  private class UniffiCleanAction implements Runnable {
    private final long handle;

    public UniffiCleanAction(long handle) {
      this.handle = handle;
    }

    @Override
    public void run() {
      // If the handle is 0 this is a fake object created with `NoHandle`, don't try to free.
      if (handle != 0L) {
        UniffiHelpers.uniffiRustCall((_allocator, status) -> {
          UniffiLib.uniffi_xcelerate_fn_free_element(handle, status);
          return null;
        });
      }
    }
  }

  long uniffiCloneHandle() {
    return UniffiHelpers.uniffiRustCall((_allocator, status) -> {
      if (handle == 0L) {
        throw new java.lang.NullPointerException();
      }
      return UniffiLib.uniffi_xcelerate_fn_clone_element(handle, status);
    });
  }

  
    /**
     * Returns the value of a specific attribute.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> attribute(java.lang.String name){
        return attribute(name, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> attribute(java.lang.String name, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_attribute(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(name)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterOptionalString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Like [`Element::call_json`] but coerces the result to a bool.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Boolean> callBool(java.lang.String function, java.lang.String argsJson){
        return callBool(function, argsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Boolean> callBool(java.lang.String function, java.lang.String argsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_call_bool(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(function), FfiConverterString.INSTANCE.lower(argsJson)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_i8(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_i8(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_i8(future),
        // lift function
        (it) -> FfiConverterBoolean.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Calls a JS function on this element with JSON-encoded arguments.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callJson(java.lang.String function, java.lang.String argsJson){
        return callJson(function, argsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callJson(java.lang.String function, java.lang.String argsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_call_json(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(function), FfiConverterString.INSTANCE.lower(argsJson)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Runs a JS function against the first descendant matching `selector`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelector(java.lang.String selector, java.lang.String expression){
        return callOnSelector(selector, expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelector(java.lang.String selector, java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_call_on_selector(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector), FfiConverterString.INSTANCE.lower(expression)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Runs a JS function against every descendant matching `selector`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelectorAll(java.lang.String selector, java.lang.String expression){
        return callOnSelectorAll(selector, expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelectorAll(java.lang.String selector, java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_call_on_selector_all(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector), FfiConverterString.INSTANCE.lower(expression)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Like [`Element::call_json`] but coerces the result to a string.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callString(java.lang.String function, java.lang.String argsJson){
        return callString(function, argsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callString(java.lang.String function, java.lang.String argsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_call_string(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(function), FfiConverterString.INSTANCE.lower(argsJson)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Clicks the element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> click(){
        return click(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> click(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_click(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Clicks the element using realistic mouse movement and CDP input events.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> clickStealth(){
        return clickStealth(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> clickStealth(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_click_stealth(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Number of elements this handle represents (always 1).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Long> count(){
        return count(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Long> count(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_count(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_i64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_i64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_i64(future),
        // lift function
        (it) -> FfiConverterLong.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Releases the underlying remote object handle.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> dispose(){
        return dispose(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> dispose(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_dispose(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Calls a function on this element and coerces the result to a bool.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Boolean> evaluateBool(java.lang.String function){
        return evaluateBool(function, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Boolean> evaluateBool(java.lang.String function, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_evaluate_bool(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(function)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_i8(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_i8(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_i8(future),
        // lift function
        (it) -> FfiConverterBoolean.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Calls a JS function on this element and returns the resulting node.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> evaluateHandle(java.lang.String function){
        return evaluateHandle(function, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> evaluateHandle(java.lang.String function, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_evaluate_handle(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(function)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Calls a function on this element; the result is returned as JSON text.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> evaluateJson(java.lang.String function){
        return evaluateJson(function, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> evaluateJson(java.lang.String function, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_evaluate_json(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(function)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Calls a function on this element and coerces the result to a string.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> evaluateString(java.lang.String function){
        return evaluateString(function, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> evaluateString(java.lang.String function, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_evaluate_string(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(function)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Focuses the element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> focus(){
        return focus(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> focus(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_focus(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Finds a descendant form control by its `<label>` text.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> getByLabel(java.lang.String label){
        return getByLabel(label, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> getByLabel(java.lang.String label, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_get_by_label(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(label)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Finds a descendant by ARIA role.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> getByRole(java.lang.String role){
        return getByRole(role, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> getByRole(java.lang.String role, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_get_by_role(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(role)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Finds a descendant whose text contains `text`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> getByText(java.lang.String text){
        return getByText(text, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> getByText(java.lang.String text, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_get_by_text(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(text)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Returns this element's enumerable properties as a JSON object.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> getProperties(){
        return getProperties(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> getProperties(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_get_properties(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Hovers over the element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> hover(){
        return hover(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> hover(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_hover(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Hovers over the element using realistic mouse movement.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> hoverStealth(){
        return hoverStealth(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> hoverStealth(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_hover_stealth(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Returns the inner HTML of the element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> innerHtml(){
        return innerHtml(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> innerHtml(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_inner_html(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Focuses the element and presses a key.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> press(java.lang.String key){
        return press(key, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> press(java.lang.String key, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_press(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(key)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Returns the first descendant matching `selector` as an [`Element`].
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> querySelector(java.lang.String selector){
        return querySelector(selector, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> querySelector(java.lang.String selector, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_query_selector(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Returns every descendant matching `selector`.
     *
     * Resolves the whole node list with a single `Runtime.getProperties` call
     * rather than one `evaluate` per match.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.util.List<Element>> querySelectorAll(java.lang.String selector){
        return querySelectorAll(selector, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.util.List<Element>> querySelectorAll(java.lang.String selector, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_query_selector_all(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterSequenceTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Finds a descendant by attribute value.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> querySelectorAttr(java.lang.String attribute, java.lang.String value){
        return querySelectorAttr(attribute, value, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> querySelectorAttr(java.lang.String attribute, java.lang.String value, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_query_selector_attr(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(attribute), FfiConverterString.INSTANCE.lower(value)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Finds a descendant matching an XPath expression.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> querySelectorXpath(java.lang.String xpath){
        return querySelectorXpath(xpath, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> querySelectorXpath(java.lang.String xpath, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_query_selector_xpath(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(xpath)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Captures a PNG screenshot cropped to this element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<byte[]> screenshot(){
        return screenshot(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<byte[]> screenshot(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_screenshot(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterByteArray.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Captures a base64 PNG screenshot cropped to this element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> screenshotBase64(){
        return screenshotBase64(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> screenshotBase64(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_screenshot_base64(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Selects options by value or label on this `<select>` element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> selectOption(java.lang.String valuesJson){
        return selectOption(valuesJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> selectOption(java.lang.String valuesJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_select_option(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(valuesJson)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Sets the files of this `<input type="file">` element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setInputFiles(java.lang.String filesJson){
        return setInputFiles(filesJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setInputFiles(java.lang.String filesJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_set_input_files(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(filesJson)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Returns the visible text of the element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> text(){
        return text(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> text(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_text(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterString.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    @Override
    public java.util.concurrent.CompletableFuture<Element> typeText(java.lang.String text){
        return typeText(text, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> typeText(java.lang.String text, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_type_text(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(text)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Waits for a descendant matching `selector` to appear.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> waitForSelector(java.lang.String selector){
        return waitForSelector(selector, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> waitForSelector(java.lang.String selector, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_element_wait_for_selector(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeElement.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  

  


  
}

