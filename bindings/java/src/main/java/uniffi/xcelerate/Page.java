package uniffi.xcelerate;

public class Page implements AutoCloseable, PageInterface {
  protected long handle;
  protected UniffiCleaner.Cleanable cleanable;

  private java.util.concurrent.atomic.AtomicBoolean wasDestroyed = new java.util.concurrent.atomic.AtomicBoolean(false);
  private java.util.concurrent.atomic.AtomicLong callCounter = new java.util.concurrent.atomic.AtomicLong(1);

  /**
   * Internal constructor to wrap a raw handle from FFI.
   * The UniffiWithHandle marker disambiguates this from other constructors.
   */
  public Page(UniffiWithHandle phantom, long handle) {
    this.handle = handle;
    this.cleanable = UniffiLib.CLEANER.register(this, new UniffiCleanAction(handle));
  }

  /**
   * This constructor can be used to instantiate a fake object. Only used for tests. Any
   * attempt to actually use an object constructed this way will fail as there is no
   * connected Rust object.
   */
  public Page(NoHandle noHandle) {
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
        throw new java.lang.IllegalStateException("Page object has already been destroyed");
      }
      if (c == java.lang.Long.MAX_VALUE) {
        throw new java.lang.IllegalStateException("Page call counter would overflow");
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
          UniffiLib.uniffi_xcelerate_fn_free_page(handle, status);
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
      return UniffiLib.uniffi_xcelerate_fn_clone_page(handle, status);
    });
  }

  
    /**
     * Activates this page's target.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> activate(){
        return activate(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> activate(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_activate(
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
     * Activates the given target (window/tab).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> activateTarget(java.lang.String targetId){
        return activateTarget(targetId, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> activateTarget(java.lang.String targetId, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_activate_target(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(targetId)
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
     * Evaluates a script on every new document.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> addScriptToEvaluateOnNewDocument(java.lang.String source){
        return addScriptToEvaluateOnNewDocument(source, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> addScriptToEvaluateOnNewDocument(java.lang.String source, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(source)
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
     * Injects a `<style>` element and returns the injected content.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> addStyleTag(java.lang.String content){
        return addStyleTag(content, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> addStyleTag(java.lang.String content, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_add_style_tag(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(content)
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
     * Sets the credentials used to answer HTTP auth challenges.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> authenticate(java.lang.String username, java.lang.String password){
        return authenticate(username, password, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> authenticate(java.lang.String username, java.lang.String password, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_authenticate(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(username), FfiConverterString.INSTANCE.lower(password)
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
     * Brings the page to the front.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> bringToFront(){
        return bringToFront(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> bringToFront(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_bring_to_front(
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
     * Like [`Page::call_json`] but coerces the result to a bool.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Boolean> callBool(java.lang.String function, java.lang.String argsJson){
        return callBool(function, argsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Boolean> callBool(java.lang.String function, java.lang.String argsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_call_bool(
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
     * Calls a JS function with JSON-encoded arguments, returning JSON text.
     *
     * This is the shim the adapters use to express the broad upstream surface
     * as data (a function body per method) rather than a core method per member.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callJson(java.lang.String function, java.lang.String argsJson){
        return callJson(function, argsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callJson(java.lang.String function, java.lang.String argsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_call_json(
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
     * Runs a JS function against the element matching `selector` (`$eval`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelector(java.lang.String selector, java.lang.String expression){
        return callOnSelector(selector, expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelector(java.lang.String selector, java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_call_on_selector(
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
     * Runs a JS function against every element matching `selector` (`$$eval`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelectorAll(java.lang.String selector, java.lang.String expression){
        return callOnSelectorAll(selector, expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callOnSelectorAll(java.lang.String selector, java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_call_on_selector_all(
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
     * Like [`Page::call_json`] but coerces the result to a string.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> callString(java.lang.String function, java.lang.String argsJson){
        return callString(function, argsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> callString(java.lang.String function, java.lang.String argsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_call_string(
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
     * Clears the recorded intercepted requests.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> clearRequests(){
        return clearRequests(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> clearRequests(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_clear_requests(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Moves the mouse to (x, y) and performs a click (down & up) with human-like delays.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Page> clickMouse(double x, double y){
        return clickMouse(x, y, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Page> clickMouse(double x, double y, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_click_mouse(
                uniffiHandle,
                x, y
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypePage.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Closes the page.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> closePage(){
        return closePage(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> closePage(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_close(
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
     * Returns the full HTML content of the page.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> content(){
        return content(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> content(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_content(
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
     * Returns a single cookie by name as JSON (or null).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> cookie(java.lang.String name){
        return cookie(name, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> cookie(java.lang.String name, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_cookie(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(name)
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
     * Returns the cookies visible to this page as a JSON array.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> cookies(){
        return cookies(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> cookies(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_cookies(
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
     * Starts CSS coverage collection.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> coverageStartCss(){
        return coverageStartCss(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> coverageStartCss(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_coverage_start_css(
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
     * Starts JS coverage collection.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> coverageStartJs(){
        return coverageStartJs(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> coverageStartJs(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_coverage_start_js(
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
     * Stops CSS coverage collection and returns the result as JSON.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> coverageStopCss(){
        return coverageStopCss(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> coverageStopCss(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_coverage_stop_css(
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
     * Stops JS coverage collection and returns the result as JSON.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> coverageStopJs(){
        return coverageStopJs(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> coverageStopJs(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_coverage_stop_js(
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
     * Returns the page PDF as a base64 string.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> createPdfStream(){
        return createPdfStream(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> createPdfStream(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_create_pdf_stream(
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
    public byte[] decodeBase64(java.lang.String data) throws XcelerateException {
            try {
                return FfiConverterByteArray.INSTANCE.lift(
    callWithHandle(uniffiHandle -> {
        try {
    
            return
    UniffiHelpers.uniffiRustCallWithError(new XcelerateExceptionErrorHandler(), (_allocator, _status) -> {
        return UniffiLib.uniffi_xcelerate_fn_method_page_decode_base64(_allocator, uniffiHandle,
            FfiConverterString.INSTANCE.lower(data), _status);
    });
    
        } catch (java.lang.Exception _uniffi_ex) {
            throw new java.lang.RuntimeException(_uniffi_ex);
        }
    })
    );
            } catch (java.lang.RuntimeException _uniffi_ex) {
                
                if (XcelerateException.class.isInstance(_uniffi_ex.getCause())) {
                    throw (XcelerateException)_uniffi_ex.getCause();
                }
                
                if (InternalException.class.isInstance(_uniffi_ex.getCause())) {
                    throw (InternalException)_uniffi_ex.getCause();
                }
                throw _uniffi_ex;
            }
    }
    

  
    /**
     * Overrides the idle state.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> emulateIdleState(boolean isUserActive, boolean isScreenUnlocked){
        return emulateIdleState(isUserActive, isScreenUnlocked, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> emulateIdleState(boolean isUserActive, boolean isScreenUnlocked, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_emulate_idle_state(
                uniffiHandle,
                FfiConverterBoolean.INSTANCE.lower(isUserActive), FfiConverterBoolean.INSTANCE.lower(isScreenUnlocked)
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
     * Emulates a media type and/or colour scheme.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> emulateMedia(java.lang.String media, java.lang.String colorScheme){
        return emulateMedia(media, colorScheme, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> emulateMedia(java.lang.String media, java.lang.String colorScheme, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_emulate_media(
                uniffiHandle,
                FfiConverterOptionalString.INSTANCE.lower(media), FfiConverterOptionalString.INSTANCE.lower(colorScheme)
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

  
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> ensureInterception(){
        return ensureInterception(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> ensureInterception(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_ensure_interception(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Evaluates JavaScript and coerces the result to a bool.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Boolean> evaluateBool(java.lang.String expression){
        return evaluateBool(expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Boolean> evaluateBool(java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_evaluate_bool(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(expression)
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
     * Evaluates JavaScript and returns the resulting object as an [`Element`].
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> evaluateHandle(java.lang.String expression){
        return evaluateHandle(expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> evaluateHandle(java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_evaluate_handle(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(expression)
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
     * Evaluates JavaScript in the page and returns the result as a JSON string.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> evaluateJson(java.lang.String expression){
        return evaluateJson(expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> evaluateJson(java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_evaluate_json(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(expression)
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
     * Evaluates JavaScript and coerces the result to a string.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> evaluateString(java.lang.String expression){
        return evaluateString(expression, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> evaluateString(java.lang.String expression, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_evaluate_string(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(expression)
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
     * Returns the registered event names.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.util.List<java.lang.String>> eventNames(){
        return eventNames(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.util.List<java.lang.String>> eventNames(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_event_names(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_rust_buffer(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_rust_buffer(_allocator, future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_rust_buffer(future),
        // lift function
        (it) -> FfiConverterSequenceString.INSTANCE.lift(it),
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Escape hatch: sends an arbitrary CDP command and returns its JSON result.
     *
     * The adapters use this to express CDP-backed library methods as data
     * (a method name + parameter template), keeping the core surface small.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> executeCdpCmd(java.lang.String method, java.lang.String paramsJson){
        return executeCdpCmd(method, paramsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> executeCdpCmd(java.lang.String method, java.lang.String paramsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_execute_cdp_cmd(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(method), FfiConverterString.INSTANCE.lower(paramsJson)
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
     * Finds an element matching the CSS selector.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> findElement(java.lang.String selector){
        return findElement(selector, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> findElement(java.lang.String selector, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_find_element(
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
     * Returns the frame matching an id or name as JSON (or null).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> frame(java.lang.String frameId){
        return frame(frameId, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> frame(java.lang.String frameId, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_frame(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(frameId)
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
     * Returns the main frame's name.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> frameName(){
        return frameName(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> frameName(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_frame_name(
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
     * Returns every frame in the page as a JSON array.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> frames(){
        return frames(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> frames(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_frames(
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
     * Finds a form control by its associated `<label>` text.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> getByLabel(java.lang.String label){
        return getByLabel(label, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> getByLabel(java.lang.String label, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_get_by_label(
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
     * Finds an element by ARIA role (falls back to a tag-name lookup).
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> getByRole(java.lang.String role){
        return getByRole(role, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> getByRole(java.lang.String role, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_get_by_role(
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
     * Finds an element whose text content contains `text`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> getByText(java.lang.String text){
        return getByText(text, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> getByText(java.lang.String text, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_get_by_text(
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
     * Returns the stored default timeout (ms).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Double> getDefaultTimeout(){
        return getDefaultTimeout(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Double> getDefaultTimeout(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_get_default_timeout(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_f64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_f64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_f64(future),
        // lift function
        (it) -> FfiConverterDouble.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> goBack(){
        return goBack(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> goBack(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_go_back(
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
     * Navigates forward in history.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> goForward(){
        return goForward(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> goForward(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_go_forward(
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
     * Accepts or dismisses the active JavaScript dialog.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> handleJsDialog(boolean accept, java.lang.String promptText){
        return handleJsDialog(accept, promptText, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> handleJsDialog(boolean accept, java.lang.String promptText, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_handle_js_dialog(
                uniffiHandle,
                FfiConverterBoolean.INSTANCE.lower(accept), FfiConverterOptionalString.INSTANCE.lower(promptText)
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
     * Reads a local file and injects it as an init script.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> injectFile(java.lang.String path){
        return injectFile(path, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> injectFile(java.lang.String path, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_inject_file(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(path)
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
     * Whether drag interception is enabled.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Boolean> isDragInterceptionEnabled(){
        return isDragInterceptionEnabled(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Boolean> isDragInterceptionEnabled(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_i8(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_i8(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_i8(future),
        // lift function
        (it) -> FfiConverterBoolean.INSTANCE.lift(it),
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Dispatches a keydown event for `key`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardDown(java.lang.String key){
        return keyboardDown(key, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardDown(java.lang.String key, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_keyboard_down(
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
     * Presses `key` on the focused element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardPress(java.lang.String key){
        return keyboardPress(key, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardPress(java.lang.String key, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_keyboard_press(
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
     * Types `text` into the focused element.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardType(java.lang.String text){
        return keyboardType(text, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardType(java.lang.String text, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_keyboard_type(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(text)
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
     * Dispatches a keyup event for `key`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardUp(java.lang.String key){
        return keyboardUp(key, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> keyboardUp(java.lang.String key, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_keyboard_up(
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
     * Whether an event name is registered.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Boolean> listensTo(java.lang.String eventName){
        return listensTo(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Boolean> listensTo(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_listens_to(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(eventName)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_i8(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_i8(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_i8(future),
        // lift function
        (it) -> FfiConverterBoolean.INSTANCE.lift(it),
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Returns the main frame as JSON.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> mainFrame(){
        return mainFrame(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> mainFrame(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_main_frame(
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
     * Returns the page performance metrics as a JSON object.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> metrics(){
        return metrics(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> metrics(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_metrics(
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
     * Triggers a mousePress event at the current mouse coordinates.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Page> mouseDown(java.lang.String button){
        return mouseDown(button, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Page> mouseDown(java.lang.String button, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_mouse_down(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(button)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypePage.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Triggers a mouseReleased event at the current mouse coordinates.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Page> mouseUp(java.lang.String button){
        return mouseUp(button, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Page> mouseUp(java.lang.String button, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_mouse_up(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(button)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypePage.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Moves the mouse cursor from the current position to the target (x, y) along a realistic Bezier curve.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Page> moveMouse(double x, double y){
        return moveMouse(x, y, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Page> moveMouse(double x, double y, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_move_mouse(
                uniffiHandle,
                x, y
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypePage.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
    /**
     * Navigates to a URL.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> navigate(java.lang.String url){
        return navigate(url, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> navigate(java.lang.String url, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_navigate(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(url)
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
     * Registers interest in a CDP event name.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> on(java.lang.String eventName){
        return on(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> on(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_on(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(eventName)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Alias for [`Page::on`].
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> once(java.lang.String eventName){
        return once(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> once(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_once(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(eventName)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    @Override
    public java.util.concurrent.CompletableFuture<byte[]> pdf(){
        return pdf(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<byte[]> pdf(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_pdf(
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
     * Focuses the element matching `selector` and presses `key`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> press(java.lang.String selector, java.lang.String key){
        return press(selector, key, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> press(java.lang.String selector, java.lang.String key, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_press(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector), FfiConverterString.INSTANCE.lower(key)
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
     * Returns every element matching the CSS selector.
     *
     * Uses two round trips (fetch the node list, then read its properties)
     * instead of one `evaluate` per match.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.util.List<Element>> querySelectorAll(java.lang.String selector){
        return querySelectorAll(selector, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.util.List<Element>> querySelectorAll(java.lang.String selector, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_query_selector_all(
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
     * Returns the first node matching an XPath expression as an [`Element`].
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> querySelectorXpath(java.lang.String xpath){
        return querySelectorXpath(xpath, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> querySelectorXpath(java.lang.String xpath, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_query_selector_xpath(
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

  
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> rawWindowBounds(){
        return rawWindowBounds(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> rawWindowBounds(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_raw_window_bounds(
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
     * Reloads the page.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> reload(){
        return reload(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> reload(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_reload(
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
     * Removes every registered event listener.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> removeAllListeners(){
        return removeAllListeners(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> removeAllListeners(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_remove_all_listeners(
                uniffiHandle
                
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Removes a single registered event listener.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> removeListener(java.lang.String eventName){
        return removeListener(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> removeListener(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_remove_listener(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(eventName)
            );
        }),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_void(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_void(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_void(future),
        // lift function
        () -> {},
        // Error FFI converter
        new UniffiNullRustCallStatusErrorHandler()
    );
    }

  
    /**
     * Removes an init script by identifier.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> removeScript(java.lang.String identifier){
        return removeScript(identifier, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> removeScript(java.lang.String identifier, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_remove_script(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(identifier)
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
     * Returns the most recent intercepted request as JSON.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> request(){
        return request(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> request(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_request(
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
     * Returns the intercepted requests seen so far as JSON.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> requests(){
        return requests(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> requests(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_requests(
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
     * Adds a route rule. `action` is `continue`, `abort`, or `fulfill`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> route(java.lang.String pattern, java.lang.String action, java.lang.String body, java.lang.String contentType){
        return route(pattern, action, body, contentType, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> route(java.lang.String pattern, java.lang.String action, java.lang.String body, java.lang.String contentType, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_route(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(pattern), FfiConverterString.INSTANCE.lower(action), FfiConverterOptionalString.INSTANCE.lower(body), FfiConverterOptionalString.INSTANCE.lower(contentType)
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
     * Aborts every request matching `pattern`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> routeAbort(java.lang.String pattern){
        return routeAbort(pattern, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> routeAbort(java.lang.String pattern, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_route_abort(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(pattern)
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
     * Registers fulfill routes for every entry in a HAR file.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> routeFromHar(java.lang.String path){
        return routeFromHar(path, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> routeFromHar(java.lang.String path, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_route_from_har(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(path)
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
     * Fulfills every request matching `pattern` with `body`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> routeFulfill(java.lang.String pattern, java.lang.String body, java.lang.String contentType){
        return routeFulfill(pattern, body, contentType, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> routeFulfill(java.lang.String pattern, java.lang.String body, java.lang.String contentType, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_route_fulfill(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(pattern), FfiConverterString.INSTANCE.lower(body), FfiConverterOptionalString.INSTANCE.lower(contentType)
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

  
    @Override
    public java.util.concurrent.CompletableFuture<byte[]> screenshot(){
        return screenshot(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<byte[]> screenshot(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_screenshot(
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

  
    @Override
    public java.util.concurrent.CompletableFuture<byte[]> screenshotFull(){
        return screenshotFull(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<byte[]> screenshotFull(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_screenshot_full(
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
     * Selects options by value/label on the matching `<select>`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> selectOption(java.lang.String selector, java.lang.String valuesJson){
        return selectOption(selector, valuesJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> selectOption(java.lang.String selector, java.lang.String valuesJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_select_option(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector), FfiConverterString.INSTANCE.lower(valuesJson)
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
     * Enables or disables the HTTP cache.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setCacheEnabled(boolean enabled){
        return setCacheEnabled(enabled, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setCacheEnabled(boolean enabled, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_cache_enabled(
                uniffiHandle,
                FfiConverterBoolean.INSTANCE.lower(enabled)
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
     * Replaces the document content.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setContent(java.lang.String html){
        return setContent(html, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setContent(java.lang.String html, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_content(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(html)
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
     * Stores a default timeout (ms) for adapter compatibility.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setDefaultTimeout(double milliseconds){
        return setDefaultTimeout(milliseconds, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setDefaultTimeout(double milliseconds, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_default_timeout(
                uniffiHandle,
                milliseconds
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
     * Enables or disables input drag interception.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setDragInterception(boolean enabled){
        return setDragInterception(enabled, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setDragInterception(boolean enabled, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_drag_interception(
                uniffiHandle,
                FfiConverterBoolean.INSTANCE.lower(enabled)
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
     * Overrides media features (JSON array of `{name,value}`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setEmulatedMediaFeatures(java.lang.String featuresJson){
        return setEmulatedMediaFeatures(featuresJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setEmulatedMediaFeatures(java.lang.String featuresJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_emulated_media_features(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(featuresJson)
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
     * Sets extra HTTP headers for every request from this page.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setExtraHttpHeaders(java.lang.String headersJson){
        return setExtraHttpHeaders(headersJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setExtraHttpHeaders(java.lang.String headersJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_extra_http_headers(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(headersJson)
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
     * Sets the files of the matching `<input type="file">`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setInputFiles(java.lang.String selector, java.lang.String filesJson){
        return setInputFiles(selector, filesJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setInputFiles(java.lang.String selector, java.lang.String filesJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_input_files(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(selector), FfiConverterString.INSTANCE.lower(filesJson)
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
     * Enables or disables JavaScript execution.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setJavascriptEnabled(boolean enabled){
        return setJavascriptEnabled(enabled, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setJavascriptEnabled(boolean enabled, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_javascript_enabled(
                uniffiHandle,
                FfiConverterBoolean.INSTANCE.lower(enabled)
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
     * Toggles offline mode.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setOffline(boolean offline){
        return setOffline(offline, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setOffline(boolean offline, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_offline(
                uniffiHandle,
                FfiConverterBoolean.INSTANCE.lower(offline)
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
     * Enables or disables request interception for this page.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setRequestInterception(boolean enabled){
        return setRequestInterception(enabled, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setRequestInterception(boolean enabled, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_request_interception(
                uniffiHandle,
                FfiConverterBoolean.INSTANCE.lower(enabled)
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
     * Restores cookies + localStorage from a storage-state JSON object.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setStorageState(java.lang.String stateJson){
        return setStorageState(stateJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setStorageState(java.lang.String stateJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_storage_state(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(stateJson)
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
     * Overrides `navigator.userAgent` for this page.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setUserAgent(java.lang.String userAgent, java.lang.String acceptLanguage){
        return setUserAgent(userAgent, acceptLanguage, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setUserAgent(java.lang.String userAgent, java.lang.String acceptLanguage, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_user_agent(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(userAgent), FfiConverterOptionalString.INSTANCE.lower(acceptLanguage)
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
     * Overrides the viewport size.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setViewportSize(long width, long height){
        return setViewportSize(width, height, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setViewportSize(long width, long height, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_viewport_size(
                uniffiHandle,
                width, height
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
     * Moves and resizes the window.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowBounds(long left, long top, long width, long height){
        return setWindowBounds(left, top, width, height, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowBounds(long left, long top, long width, long height, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_window_bounds(
                uniffiHandle,
                left, top, width, height
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
     * Moves the window, preserving its size.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowPosition(long x, long y){
        return setWindowPosition(x, y, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowPosition(long x, long y, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_window_position(
                uniffiHandle,
                x, y
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
     * Resizes the window, preserving its position.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowSize(long width, long height){
        return setWindowSize(width, height, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowSize(long width, long height, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_window_size(
                uniffiHandle,
                width, height
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
     * Sets the window state (`normal` | `minimized` | `maximized` | `fullscreen`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowState(java.lang.String state){
        return setWindowState(state, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setWindowState(java.lang.String state, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_set_window_state(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(state)
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
     * Starts a PNG screencast.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> startScreencast(){
        return startScreencast(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> startScreencast(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_start_screencast(
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
     * Starts CDP tracing on this page's session.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> startTracing(){
        return startTracing(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> startTracing(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_start_tracing(
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
     * Stops the screencast.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> stopScreencast(){
        return stopScreencast(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> stopScreencast(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_stop_screencast(
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
     * Stops CDP tracing on this page's session.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> stopTracing(){
        return stopTracing(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> stopTracing(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_stop_tracing(
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
     * Returns cookies + localStorage as a storage-state JSON object.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> storageState(){
        return storageState(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> storageState(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_storage_state(
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
     * The CDP target id backing this page.
     */
    @Override
    public java.lang.String targetId()  {
            try {
                return FfiConverterString.INSTANCE.lift(
    callWithHandle(uniffiHandle -> {
        try {
    
            return
    UniffiHelpers.uniffiRustCall( (_allocator, _status) -> {
        return UniffiLib.uniffi_xcelerate_fn_method_page_target_id(_allocator, uniffiHandle,
            _status);
    });
    
        } catch (java.lang.Exception _uniffi_ex) {
            throw new java.lang.RuntimeException(_uniffi_ex);
        }
    })
    );
            } catch (java.lang.RuntimeException _uniffi_ex) {
                
                
                if (InternalException.class.isInstance(_uniffi_ex.getCause())) {
                    throw (InternalException)_uniffi_ex.getCause();
                }
                throw _uniffi_ex;
            }
    }
    

  
    /**
     * Returns the page title.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> title(){
        return title(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> title(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_title(
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
     * Dispatches a touch tap at (x, y).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> touchTap(double x, double y){
        return touchTap(x, y, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> touchTap(double x, double y, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_touch_tap(
                uniffiHandle,
                x, y
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
     * Removes the routes registered for `pattern`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> unroute(java.lang.String pattern){
        return unroute(pattern, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> unroute(java.lang.String pattern, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_unroute(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(pattern)
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
     * Removes every route rule.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> unrouteAll(){
        return unrouteAll(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> unrouteAll(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_unroute_all(
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
     * Returns the current document URL.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> url(){
        return url(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> url(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_url(
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
     * Waits for the next CDP event named `event_name` and returns its params.
     *
     * The relevant domain is enabled first (best effort), so callers do not
     * have to.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> waitForEvent(java.lang.String eventName, long timeoutMs){
        return waitForEvent(eventName, timeoutMs, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> waitForEvent(java.lang.String eventName, long timeoutMs, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_wait_for_event(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(eventName), timeoutMs
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
     * [`Page::wait_for_event`] with the default 30s timeout.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> waitForEventDefault(java.lang.String eventName){
        return waitForEventDefault(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> waitForEventDefault(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_wait_for_event_default(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(eventName)
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
     * Polls `expression` until it evaluates truthy or `timeout_ms` elapses.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> waitForFunction(java.lang.String expression, long timeoutMs){
        return waitForFunction(expression, timeoutMs, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> waitForFunction(java.lang.String expression, long timeoutMs, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_wait_for_function(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(expression), timeoutMs
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
     * Waits for the page to finish loading.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> waitForNavigation(){
        return waitForNavigation(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> waitForNavigation(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_wait_for_navigation(
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
     * Waits for an element matching the selector to appear in the DOM.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> waitForSelector(java.lang.String selector){
        return waitForSelector(selector, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> waitForSelector(java.lang.String selector, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_wait_for_selector(
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
     * Waits for the first XPath match to appear.
     */
    @Override
    public java.util.concurrent.CompletableFuture<Element> waitForXpath(java.lang.String xpath, long timeoutMs){
        return waitForXpath(xpath, timeoutMs, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Element> waitForXpath(java.lang.String xpath, long timeoutMs, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_wait_for_xpath(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(xpath), timeoutMs
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

  
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Long> windowId(){
        return windowId(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Long> windowId(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_window_id(
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
     * Returns the window position as JSON (`{x,y}`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> windowPosition(){
        return windowPosition(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> windowPosition(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_window_position(
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
     * Returns the window bounds as JSON (`{left,top,width,height,windowState}`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> windowRect(){
        return windowRect(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> windowRect(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_window_rect(
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
     * Returns the window size as JSON (`{width,height}`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> windowSize(){
        return windowSize(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> windowSize(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_page_window_size(
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

  

  


  
}

