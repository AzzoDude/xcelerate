package uniffi.xcelerate;

/**
 * Represents a browser instance (e.g., Chrome or Edge).
 */
public class Browser implements AutoCloseable, BrowserInterface {
  protected long handle;
  protected UniffiCleaner.Cleanable cleanable;

  private java.util.concurrent.atomic.AtomicBoolean wasDestroyed = new java.util.concurrent.atomic.AtomicBoolean(false);
  private java.util.concurrent.atomic.AtomicLong callCounter = new java.util.concurrent.atomic.AtomicLong(1);

  /**
   * Internal constructor to wrap a raw handle from FFI.
   * The UniffiWithHandle marker disambiguates this from other constructors.
   */
  public Browser(UniffiWithHandle phantom, long handle) {
    this.handle = handle;
    this.cleanable = UniffiLib.CLEANER.register(this, new UniffiCleanAction(handle));
  }

  /**
   * This constructor can be used to instantiate a fake object. Only used for tests. Any
   * attempt to actually use an object constructed this way will fail as there is no
   * connected Rust object.
   */
  public Browser(NoHandle noHandle) {
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
        throw new java.lang.IllegalStateException("Browser object has already been destroyed");
      }
      if (c == java.lang.Long.MAX_VALUE) {
        throw new java.lang.IllegalStateException("Browser call counter would overflow");
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
          UniffiLib.uniffi_xcelerate_fn_free_browser(handle, status);
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
      return UniffiLib.uniffi_xcelerate_fn_clone_browser(handle, status);
    });
  }

  
    /**
     * Returns the browser context ids as a JSON array.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> browserContexts(){
        return browserContexts(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> browserContexts(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_browser_contexts(
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
     * Returns the browser version info as JSON.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> capabilities(){
        return capabilities(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> capabilities(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_capabilities(
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
     * Closes the browser and kills the process.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> closeBrowser(){
        return closeBrowser(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> closeBrowser(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_close(
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
     * Returns all browser cookies as a JSON array.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> cookies(){
        return cookies(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> cookies(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_cookies(
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
     * Deletes cookies with the given name.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> deleteCookie(java.lang.String name){
        return deleteCookie(name, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> deleteCookie(java.lang.String name, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_delete_cookie(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(name)
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
            return UniffiLib.uniffi_xcelerate_fn_method_browser_event_names(
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
     * Grants permissions (JSON array) to an origin.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> grantPermissions(java.lang.String origin, java.lang.String permissionsJson){
        return grantPermissions(origin, permissionsJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> grantPermissions(java.lang.String origin, java.lang.String permissionsJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_grant_permissions(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(origin), FfiConverterString.INSTANCE.lower(permissionsJson)
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
     * Whether the underlying connection is alive.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Boolean> isConnected(){
        return isConnected(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Boolean> isConnected(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_is_connected(
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
            return UniffiLib.uniffi_xcelerate_fn_method_browser_listens_to(
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
     * Creates a new (incognito) browser context and returns its id.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> newContext(){
        return newContext(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> newContext(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_new_context(
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
    public java.util.concurrent.CompletableFuture<Page> newPage(java.lang.String url){
        return newPage(url, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<Page> newPage(java.lang.String url, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_new_page(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(url)
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
     * Registers interest in a root-session CDP event.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> on(java.lang.String eventName){
        return on(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> on(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_on(
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
     * Alias for [`Browser::on`].
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> once(java.lang.String eventName){
        return once(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> once(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_once(
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
     * Removes every registered listener.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> removeAllListeners(){
        return removeAllListeners(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> removeAllListeners(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_remove_all_listeners(
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
     * Removes a single registered listener.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> removeListener(java.lang.String eventName){
        return removeListener(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> removeListener(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_remove_listener(
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
     * Resets all permission overrides.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> resetPermissions(){
        return resetPermissions(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> resetPermissions(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_reset_permissions(
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
     * Sets a cookie from a JSON object.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setCookie(java.lang.String cookieJson){
        return setCookie(cookieJson, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setCookie(java.lang.String cookieJson, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_set_cookie(
                uniffiHandle,
                FfiConverterString.INSTANCE.lower(cookieJson)
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
     * Sets the download directory for the browser.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> setDownloadBehavior(java.lang.String path){
        return setDownloadBehavior(path, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> setDownloadBehavior(java.lang.String path, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_set_download_behavior(
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
     * Starts CDP tracing.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> startTracing(){
        return startTracing(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> startTracing(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_start_tracing(
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
     * Stops CDP tracing.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.Void> stopTracing(){
        return stopTracing(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.Void> stopTracing(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_stop_tracing(
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
     * Returns the current targets as a JSON array (`Target.getTargets`).
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> targets(){
        return targets(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> targets(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_targets(
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
     * Returns the browser's user agent.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> userAgent(){
        return userAgent(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> userAgent(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_user_agent(
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
     * Returns the browser version information.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> version(){
        return version(java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> version(java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_version(
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
     * Waits for the next root-session CDP event named `event_name`.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> waitForEvent(java.lang.String eventName, long timeoutMs){
        return waitForEvent(eventName, timeoutMs, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> waitForEvent(java.lang.String eventName, long timeoutMs, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_wait_for_event(
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
     * [`Browser::wait_for_event`] with the default 30s timeout.
     */
    @Override
    public java.util.concurrent.CompletableFuture<java.lang.String> waitForEventDefault(java.lang.String eventName){
        return waitForEventDefault(eventName, java.util.concurrent.ForkJoinPool.commonPool());
    }public java.util.concurrent.CompletableFuture<java.lang.String> waitForEventDefault(java.lang.String eventName, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        callWithHandle(uniffiHandle -> {
            return UniffiLib.uniffi_xcelerate_fn_method_browser_wait_for_event_default(
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
     * The WebSocket endpoint Chrome was launched with.
     */
    @Override
    public java.lang.String wsEndpoint()  {
            try {
                return FfiConverterString.INSTANCE.lift(
    callWithHandle(uniffiHandle -> {
        try {
    
            return
    UniffiHelpers.uniffiRustCall( (_allocator, _status) -> {
        return UniffiLib.uniffi_xcelerate_fn_method_browser_ws_endpoint(_allocator, uniffiHandle,
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
    

  

  


  public static java.util.concurrent.CompletableFuture<Browser> launch(BrowserConfig config){
        return launch(config, java.util.concurrent.ForkJoinPool.commonPool());
    }public static java.util.concurrent.CompletableFuture<Browser> launch(BrowserConfig config, java.util.concurrent.Executor uniffiExecutor
    ){
        return UniffiAsyncHelpers.uniffiRustCallAsync(
        uniffiExecutor,
        UniffiLib.uniffi_xcelerate_fn_constructor_browser_launch(FfiConverterTypeBrowserConfig.INSTANCE.lower(config)),
        (future, callback, continuationHandle) -> UniffiLib.ffi_xcelerate_rust_future_poll_u64(future, callback, continuationHandle),
        (_allocator, future, continuation) -> UniffiLib.ffi_xcelerate_rust_future_complete_u64(future, continuation),
        (future) -> UniffiLib.ffi_xcelerate_rust_future_free_u64(future),
        // lift function
        (it) -> FfiConverterTypeBrowser.INSTANCE.lift(it),
        // Error FFI converter
        new XcelerateExceptionErrorHandler()
    );
    }

  
  
}

