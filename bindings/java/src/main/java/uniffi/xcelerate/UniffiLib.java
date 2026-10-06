package uniffi.xcelerate;


// FFM-based library binding. Each FFI function gets a MethodHandle and a wrapper method.
final class UniffiLib {
    private static final java.lang.foreign.Linker LINKER = java.lang.foreign.Linker.nativeLinker();
    private static final java.lang.foreign.SymbolLookup SYMBOLS;

    
    // The Cleaner for the whole library
    static UniffiCleaner CLEANER;

    static {
        SYMBOLS = NamespaceLibrary.loadLibrary();
    }

    private static java.lang.invoke.MethodHandle findDowncallHandle(String name, java.lang.foreign.FunctionDescriptor descriptor) {
        return SYMBOLS.find(name)
            .map(s -> LINKER.downcallHandle(s, descriptor))
            .orElseThrow(() -> new RuntimeException("Missing FFI symbol: " + name));
    }

    // uniffi_xcelerate_fn_clone_browser
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_clone_browser = findDowncallHandle("uniffi_xcelerate_fn_clone_browser", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static long uniffi_xcelerate_fn_clone_browser(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_clone_browser.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_free_browser
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_free_browser = findDowncallHandle("uniffi_xcelerate_fn_free_browser", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static void uniffi_xcelerate_fn_free_browser(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            MH_uniffi_xcelerate_fn_free_browser.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_constructor_browser_launch
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_constructor_browser_launch = findDowncallHandle("uniffi_xcelerate_fn_constructor_browser_launch", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_constructor_browser_launch(java.lang.foreign.MemorySegment config) {
        try {
            return (long) MH_uniffi_xcelerate_fn_constructor_browser_launch.invokeExact(config);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_audit_log
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_audit_log = findDowncallHandle("uniffi_xcelerate_fn_method_browser_audit_log", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_browser_audit_log(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_browser_audit_log.invokeExact(_allocator, ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_audit_verify
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_audit_verify = findDowncallHandle("uniffi_xcelerate_fn_method_browser_audit_verify", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_BYTE, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static byte uniffi_xcelerate_fn_method_browser_audit_verify(long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (byte) MH_uniffi_xcelerate_fn_method_browser_audit_verify.invokeExact(ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_available_plugins
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_available_plugins = findDowncallHandle("uniffi_xcelerate_fn_method_browser_available_plugins", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_browser_available_plugins(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_browser_available_plugins.invokeExact(_allocator, ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_browser_contexts
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_browser_contexts = findDowncallHandle("uniffi_xcelerate_fn_method_browser_browser_contexts", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_browser_contexts(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_browser_contexts.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_capabilities
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_capabilities = findDowncallHandle("uniffi_xcelerate_fn_method_browser_capabilities", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_capabilities(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_capabilities.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_close
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_close = findDowncallHandle("uniffi_xcelerate_fn_method_browser_close", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_close(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_close.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_cookies
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_cookies = findDowncallHandle("uniffi_xcelerate_fn_method_browser_cookies", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_cookies(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_cookies.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_delete_cookie
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_delete_cookie = findDowncallHandle("uniffi_xcelerate_fn_method_browser_delete_cookie", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_delete_cookie(long ptr, java.lang.foreign.MemorySegment name) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_delete_cookie.invokeExact(ptr, name);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_event_names
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_event_names = findDowncallHandle("uniffi_xcelerate_fn_method_browser_event_names", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_event_names(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_event_names.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_grant_permissions
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_grant_permissions = findDowncallHandle("uniffi_xcelerate_fn_method_browser_grant_permissions", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_grant_permissions(long ptr, java.lang.foreign.MemorySegment origin, java.lang.foreign.MemorySegment permissionsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_grant_permissions.invokeExact(ptr, origin, permissionsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_is_connected
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_is_connected = findDowncallHandle("uniffi_xcelerate_fn_method_browser_is_connected", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_is_connected(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_is_connected.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_listens_to
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_listens_to = findDowncallHandle("uniffi_xcelerate_fn_method_browser_listens_to", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_listens_to(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_listens_to.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_load_plugin
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_load_plugin = findDowncallHandle("uniffi_xcelerate_fn_method_browser_load_plugin", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_browser_load_plugin(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment path, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_browser_load_plugin.invokeExact(_allocator, ptr, path, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_new_context
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_new_context = findDowncallHandle("uniffi_xcelerate_fn_method_browser_new_context", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_new_context(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_new_context.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_new_page
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_new_page = findDowncallHandle("uniffi_xcelerate_fn_method_browser_new_page", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_new_page(long ptr, java.lang.foreign.MemorySegment url) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_new_page.invokeExact(ptr, url);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_on
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_on = findDowncallHandle("uniffi_xcelerate_fn_method_browser_on", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_on(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_on.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_once
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_once = findDowncallHandle("uniffi_xcelerate_fn_method_browser_once", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_once(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_once.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_plugin
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_plugin = findDowncallHandle("uniffi_xcelerate_fn_method_browser_plugin", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.ADDRESS));

    static long uniffi_xcelerate_fn_method_browser_plugin(long ptr, java.lang.foreign.MemorySegment name, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_plugin.invokeExact(ptr, name, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_plugin_names
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_plugin_names = findDowncallHandle("uniffi_xcelerate_fn_method_browser_plugin_names", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_browser_plugin_names(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_browser_plugin_names.invokeExact(_allocator, ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_remove_all_listeners
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_remove_all_listeners = findDowncallHandle("uniffi_xcelerate_fn_method_browser_remove_all_listeners", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_remove_all_listeners(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_remove_all_listeners.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_remove_listener
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_remove_listener = findDowncallHandle("uniffi_xcelerate_fn_method_browser_remove_listener", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_remove_listener(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_remove_listener.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_reset_permissions
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_reset_permissions = findDowncallHandle("uniffi_xcelerate_fn_method_browser_reset_permissions", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_reset_permissions(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_reset_permissions.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_set_cookie
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_set_cookie = findDowncallHandle("uniffi_xcelerate_fn_method_browser_set_cookie", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_set_cookie(long ptr, java.lang.foreign.MemorySegment cookieJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_set_cookie.invokeExact(ptr, cookieJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_set_download_behavior
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_set_download_behavior = findDowncallHandle("uniffi_xcelerate_fn_method_browser_set_download_behavior", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_set_download_behavior(long ptr, java.lang.foreign.MemorySegment path) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_set_download_behavior.invokeExact(ptr, path);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_start_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_start_tracing = findDowncallHandle("uniffi_xcelerate_fn_method_browser_start_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_start_tracing(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_start_tracing.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_stop_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_stop_tracing = findDowncallHandle("uniffi_xcelerate_fn_method_browser_stop_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_stop_tracing(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_stop_tracing.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_targets
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_targets = findDowncallHandle("uniffi_xcelerate_fn_method_browser_targets", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_targets(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_targets.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_use_plugin
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_use_plugin = findDowncallHandle("uniffi_xcelerate_fn_method_browser_use_plugin", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_use_plugin(long ptr, java.lang.foreign.MemorySegment name) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_use_plugin.invokeExact(ptr, name);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_user_agent
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_user_agent = findDowncallHandle("uniffi_xcelerate_fn_method_browser_user_agent", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_user_agent(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_user_agent.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_version
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_version = findDowncallHandle("uniffi_xcelerate_fn_method_browser_version", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_version(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_version.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_wait_for_event
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_wait_for_event = findDowncallHandle("uniffi_xcelerate_fn_method_browser_wait_for_event", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_browser_wait_for_event(long ptr, java.lang.foreign.MemorySegment eventName, long timeoutMs) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_wait_for_event.invokeExact(ptr, eventName, timeoutMs);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_wait_for_event_default
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_wait_for_event_default = findDowncallHandle("uniffi_xcelerate_fn_method_browser_wait_for_event_default", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_browser_wait_for_event_default(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_browser_wait_for_event_default.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_browser_ws_endpoint
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_browser_ws_endpoint = findDowncallHandle("uniffi_xcelerate_fn_method_browser_ws_endpoint", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_browser_ws_endpoint(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_browser_ws_endpoint.invokeExact(_allocator, ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_clone_element
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_clone_element = findDowncallHandle("uniffi_xcelerate_fn_clone_element", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static long uniffi_xcelerate_fn_clone_element(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_clone_element.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_free_element
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_free_element = findDowncallHandle("uniffi_xcelerate_fn_free_element", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static void uniffi_xcelerate_fn_free_element(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            MH_uniffi_xcelerate_fn_free_element.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_attribute
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_attribute = findDowncallHandle("uniffi_xcelerate_fn_method_element_attribute", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_attribute(long ptr, java.lang.foreign.MemorySegment name) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_attribute.invokeExact(ptr, name);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_call_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_call_bool = findDowncallHandle("uniffi_xcelerate_fn_method_element_call_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_call_bool(long ptr, java.lang.foreign.MemorySegment function, java.lang.foreign.MemorySegment argsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_call_bool.invokeExact(ptr, function, argsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_call_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_call_json = findDowncallHandle("uniffi_xcelerate_fn_method_element_call_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_call_json(long ptr, java.lang.foreign.MemorySegment function, java.lang.foreign.MemorySegment argsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_call_json.invokeExact(ptr, function, argsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_call_on_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_call_on_selector = findDowncallHandle("uniffi_xcelerate_fn_method_element_call_on_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_call_on_selector(long ptr, java.lang.foreign.MemorySegment selector, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_call_on_selector.invokeExact(ptr, selector, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_call_on_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_call_on_selector_all = findDowncallHandle("uniffi_xcelerate_fn_method_element_call_on_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_call_on_selector_all(long ptr, java.lang.foreign.MemorySegment selector, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_call_on_selector_all.invokeExact(ptr, selector, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_call_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_call_string = findDowncallHandle("uniffi_xcelerate_fn_method_element_call_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_call_string(long ptr, java.lang.foreign.MemorySegment function, java.lang.foreign.MemorySegment argsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_call_string.invokeExact(ptr, function, argsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_click
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_click = findDowncallHandle("uniffi_xcelerate_fn_method_element_click", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_click(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_click.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_click_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_click_mouse = findDowncallHandle("uniffi_xcelerate_fn_method_element_click_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_click_mouse(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_click_mouse.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_count
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_count = findDowncallHandle("uniffi_xcelerate_fn_method_element_count", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_count(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_count.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_dispose
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_dispose = findDowncallHandle("uniffi_xcelerate_fn_method_element_dispose", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_dispose(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_dispose.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_evaluate_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_evaluate_bool = findDowncallHandle("uniffi_xcelerate_fn_method_element_evaluate_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_evaluate_bool(long ptr, java.lang.foreign.MemorySegment function) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_evaluate_bool.invokeExact(ptr, function);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_evaluate_handle
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_evaluate_handle = findDowncallHandle("uniffi_xcelerate_fn_method_element_evaluate_handle", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_evaluate_handle(long ptr, java.lang.foreign.MemorySegment function) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_evaluate_handle.invokeExact(ptr, function);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_evaluate_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_evaluate_json = findDowncallHandle("uniffi_xcelerate_fn_method_element_evaluate_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_evaluate_json(long ptr, java.lang.foreign.MemorySegment function) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_evaluate_json.invokeExact(ptr, function);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_evaluate_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_evaluate_string = findDowncallHandle("uniffi_xcelerate_fn_method_element_evaluate_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_evaluate_string(long ptr, java.lang.foreign.MemorySegment function) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_evaluate_string.invokeExact(ptr, function);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_focus
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_focus = findDowncallHandle("uniffi_xcelerate_fn_method_element_focus", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_focus(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_focus.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_get_by_label
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_get_by_label = findDowncallHandle("uniffi_xcelerate_fn_method_element_get_by_label", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_get_by_label(long ptr, java.lang.foreign.MemorySegment label) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_get_by_label.invokeExact(ptr, label);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_get_by_role
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_get_by_role = findDowncallHandle("uniffi_xcelerate_fn_method_element_get_by_role", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_get_by_role(long ptr, java.lang.foreign.MemorySegment role) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_get_by_role.invokeExact(ptr, role);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_get_by_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_get_by_text = findDowncallHandle("uniffi_xcelerate_fn_method_element_get_by_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_get_by_text(long ptr, java.lang.foreign.MemorySegment text) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_get_by_text.invokeExact(ptr, text);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_get_properties
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_get_properties = findDowncallHandle("uniffi_xcelerate_fn_method_element_get_properties", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_get_properties(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_get_properties.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_hover
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_hover = findDowncallHandle("uniffi_xcelerate_fn_method_element_hover", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_hover(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_hover.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_hover_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_hover_mouse = findDowncallHandle("uniffi_xcelerate_fn_method_element_hover_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_hover_mouse(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_hover_mouse.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_inner_html
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_inner_html = findDowncallHandle("uniffi_xcelerate_fn_method_element_inner_html", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_inner_html(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_inner_html.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_press
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_press = findDowncallHandle("uniffi_xcelerate_fn_method_element_press", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_press(long ptr, java.lang.foreign.MemorySegment key) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_press.invokeExact(ptr, key);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_query_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_query_selector = findDowncallHandle("uniffi_xcelerate_fn_method_element_query_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_query_selector(long ptr, java.lang.foreign.MemorySegment selector) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_query_selector.invokeExact(ptr, selector);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_query_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_query_selector_all = findDowncallHandle("uniffi_xcelerate_fn_method_element_query_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_query_selector_all(long ptr, java.lang.foreign.MemorySegment selector) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_query_selector_all.invokeExact(ptr, selector);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_query_selector_attr
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_query_selector_attr = findDowncallHandle("uniffi_xcelerate_fn_method_element_query_selector_attr", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_query_selector_attr(long ptr, java.lang.foreign.MemorySegment attribute, java.lang.foreign.MemorySegment value) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_query_selector_attr.invokeExact(ptr, attribute, value);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_query_selector_xpath
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_query_selector_xpath = findDowncallHandle("uniffi_xcelerate_fn_method_element_query_selector_xpath", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_query_selector_xpath(long ptr, java.lang.foreign.MemorySegment xpath) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_query_selector_xpath.invokeExact(ptr, xpath);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_screenshot
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_screenshot = findDowncallHandle("uniffi_xcelerate_fn_method_element_screenshot", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_screenshot(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_screenshot.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_screenshot_base64
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_screenshot_base64 = findDowncallHandle("uniffi_xcelerate_fn_method_element_screenshot_base64", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_screenshot_base64(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_screenshot_base64.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_select_option
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_select_option = findDowncallHandle("uniffi_xcelerate_fn_method_element_select_option", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_select_option(long ptr, java.lang.foreign.MemorySegment valuesJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_select_option.invokeExact(ptr, valuesJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_set_input_files
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_set_input_files = findDowncallHandle("uniffi_xcelerate_fn_method_element_set_input_files", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_set_input_files(long ptr, java.lang.foreign.MemorySegment filesJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_set_input_files.invokeExact(ptr, filesJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_text = findDowncallHandle("uniffi_xcelerate_fn_method_element_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_element_text(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_text.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_type_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_type_text = findDowncallHandle("uniffi_xcelerate_fn_method_element_type_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_type_text(long ptr, java.lang.foreign.MemorySegment text) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_type_text.invokeExact(ptr, text);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_element_wait_for_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_element_wait_for_selector = findDowncallHandle("uniffi_xcelerate_fn_method_element_wait_for_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_element_wait_for_selector(long ptr, java.lang.foreign.MemorySegment selector) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_element_wait_for_selector.invokeExact(ptr, selector);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_clone_page
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_clone_page = findDowncallHandle("uniffi_xcelerate_fn_clone_page", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static long uniffi_xcelerate_fn_clone_page(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_clone_page.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_free_page
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_free_page = findDowncallHandle("uniffi_xcelerate_fn_free_page", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static void uniffi_xcelerate_fn_free_page(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            MH_uniffi_xcelerate_fn_free_page.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_activate
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_activate = findDowncallHandle("uniffi_xcelerate_fn_method_page_activate", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_activate(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_activate.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_activate_target
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_activate_target = findDowncallHandle("uniffi_xcelerate_fn_method_page_activate_target", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_activate_target(long ptr, java.lang.foreign.MemorySegment targetId) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_activate_target.invokeExact(ptr, targetId);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document = findDowncallHandle("uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document(long ptr, java.lang.foreign.MemorySegment source) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_add_script_to_evaluate_on_new_document.invokeExact(ptr, source);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_add_style_tag
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_add_style_tag = findDowncallHandle("uniffi_xcelerate_fn_method_page_add_style_tag", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_add_style_tag(long ptr, java.lang.foreign.MemorySegment content) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_add_style_tag.invokeExact(ptr, content);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_authenticate
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_authenticate = findDowncallHandle("uniffi_xcelerate_fn_method_page_authenticate", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_authenticate(long ptr, java.lang.foreign.MemorySegment username, java.lang.foreign.MemorySegment password) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_authenticate.invokeExact(ptr, username, password);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_bring_to_front
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_bring_to_front = findDowncallHandle("uniffi_xcelerate_fn_method_page_bring_to_front", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_bring_to_front(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_bring_to_front.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_call_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_call_bool = findDowncallHandle("uniffi_xcelerate_fn_method_page_call_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_call_bool(long ptr, java.lang.foreign.MemorySegment function, java.lang.foreign.MemorySegment argsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_call_bool.invokeExact(ptr, function, argsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_call_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_call_json = findDowncallHandle("uniffi_xcelerate_fn_method_page_call_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_call_json(long ptr, java.lang.foreign.MemorySegment function, java.lang.foreign.MemorySegment argsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_call_json.invokeExact(ptr, function, argsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_call_on_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_call_on_selector = findDowncallHandle("uniffi_xcelerate_fn_method_page_call_on_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_call_on_selector(long ptr, java.lang.foreign.MemorySegment selector, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_call_on_selector.invokeExact(ptr, selector, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_call_on_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_call_on_selector_all = findDowncallHandle("uniffi_xcelerate_fn_method_page_call_on_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_call_on_selector_all(long ptr, java.lang.foreign.MemorySegment selector, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_call_on_selector_all.invokeExact(ptr, selector, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_call_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_call_string = findDowncallHandle("uniffi_xcelerate_fn_method_page_call_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_call_string(long ptr, java.lang.foreign.MemorySegment function, java.lang.foreign.MemorySegment argsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_call_string.invokeExact(ptr, function, argsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_clear_requests
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_clear_requests = findDowncallHandle("uniffi_xcelerate_fn_method_page_clear_requests", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_clear_requests(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_clear_requests.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_click_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_click_mouse = findDowncallHandle("uniffi_xcelerate_fn_method_page_click_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_DOUBLE, java.lang.foreign.ValueLayout.JAVA_DOUBLE));

    static long uniffi_xcelerate_fn_method_page_click_mouse(long ptr, double x, double y) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_click_mouse.invokeExact(ptr, x, y);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_close
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_close = findDowncallHandle("uniffi_xcelerate_fn_method_page_close", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_close(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_close.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_content
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_content = findDowncallHandle("uniffi_xcelerate_fn_method_page_content", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_content(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_content.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_cookie
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_cookie = findDowncallHandle("uniffi_xcelerate_fn_method_page_cookie", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_cookie(long ptr, java.lang.foreign.MemorySegment name) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_cookie.invokeExact(ptr, name);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_cookies
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_cookies = findDowncallHandle("uniffi_xcelerate_fn_method_page_cookies", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_cookies(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_cookies.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_coverage_start_css
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_coverage_start_css = findDowncallHandle("uniffi_xcelerate_fn_method_page_coverage_start_css", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_coverage_start_css(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_coverage_start_css.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_coverage_start_js
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_coverage_start_js = findDowncallHandle("uniffi_xcelerate_fn_method_page_coverage_start_js", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_coverage_start_js(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_coverage_start_js.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_coverage_stop_css
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_coverage_stop_css = findDowncallHandle("uniffi_xcelerate_fn_method_page_coverage_stop_css", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_coverage_stop_css(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_coverage_stop_css.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_coverage_stop_js
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_coverage_stop_js = findDowncallHandle("uniffi_xcelerate_fn_method_page_coverage_stop_js", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_coverage_stop_js(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_coverage_stop_js.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_create_pdf_stream
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_create_pdf_stream = findDowncallHandle("uniffi_xcelerate_fn_method_page_create_pdf_stream", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_create_pdf_stream(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_create_pdf_stream.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_decode_base64
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_decode_base64 = findDowncallHandle("uniffi_xcelerate_fn_method_page_decode_base64", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_page_decode_base64(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment data, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_page_decode_base64.invokeExact(_allocator, ptr, data, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_default_timeout
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_default_timeout = findDowncallHandle("uniffi_xcelerate_fn_method_page_default_timeout", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static long uniffi_xcelerate_fn_method_page_default_timeout(long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_default_timeout.invokeExact(ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_document_element
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_document_element = findDowncallHandle("uniffi_xcelerate_fn_method_page_document_element", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_document_element(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_document_element.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_emulate_idle_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_emulate_idle_state = findDowncallHandle("uniffi_xcelerate_fn_method_page_emulate_idle_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_BYTE, java.lang.foreign.ValueLayout.JAVA_BYTE));

    static long uniffi_xcelerate_fn_method_page_emulate_idle_state(long ptr, byte isUserActive, byte isScreenUnlocked) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_emulate_idle_state.invokeExact(ptr, isUserActive, isScreenUnlocked);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_emulate_media
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_emulate_media = findDowncallHandle("uniffi_xcelerate_fn_method_page_emulate_media", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_emulate_media(long ptr, java.lang.foreign.MemorySegment media, java.lang.foreign.MemorySegment colorScheme) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_emulate_media.invokeExact(ptr, media, colorScheme);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_ensure_interception
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_ensure_interception = findDowncallHandle("uniffi_xcelerate_fn_method_page_ensure_interception", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_ensure_interception(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_ensure_interception.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_evaluate_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_evaluate_bool = findDowncallHandle("uniffi_xcelerate_fn_method_page_evaluate_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_evaluate_bool(long ptr, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_evaluate_bool.invokeExact(ptr, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_evaluate_handle
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_evaluate_handle = findDowncallHandle("uniffi_xcelerate_fn_method_page_evaluate_handle", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_evaluate_handle(long ptr, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_evaluate_handle.invokeExact(ptr, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_evaluate_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_evaluate_json = findDowncallHandle("uniffi_xcelerate_fn_method_page_evaluate_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_evaluate_json(long ptr, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_evaluate_json.invokeExact(ptr, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_evaluate_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_evaluate_string = findDowncallHandle("uniffi_xcelerate_fn_method_page_evaluate_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_evaluate_string(long ptr, java.lang.foreign.MemorySegment expression) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_evaluate_string.invokeExact(ptr, expression);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_event_names
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_event_names = findDowncallHandle("uniffi_xcelerate_fn_method_page_event_names", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_event_names(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_event_names.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_execute_cdp_cmd
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_execute_cdp_cmd = findDowncallHandle("uniffi_xcelerate_fn_method_page_execute_cdp_cmd", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_execute_cdp_cmd(long ptr, java.lang.foreign.MemorySegment method, java.lang.foreign.MemorySegment paramsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_execute_cdp_cmd.invokeExact(ptr, method, paramsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_find_element
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_find_element = findDowncallHandle("uniffi_xcelerate_fn_method_page_find_element", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_find_element(long ptr, java.lang.foreign.MemorySegment selector) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_find_element.invokeExact(ptr, selector);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_frame
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_frame = findDowncallHandle("uniffi_xcelerate_fn_method_page_frame", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_frame(long ptr, java.lang.foreign.MemorySegment frameId) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_frame.invokeExact(ptr, frameId);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_frame_name
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_frame_name = findDowncallHandle("uniffi_xcelerate_fn_method_page_frame_name", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_frame_name(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_frame_name.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_frames
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_frames = findDowncallHandle("uniffi_xcelerate_fn_method_page_frames", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_frames(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_frames.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_get_by_label
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_get_by_label = findDowncallHandle("uniffi_xcelerate_fn_method_page_get_by_label", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_get_by_label(long ptr, java.lang.foreign.MemorySegment label) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_get_by_label.invokeExact(ptr, label);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_get_by_role
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_get_by_role = findDowncallHandle("uniffi_xcelerate_fn_method_page_get_by_role", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_get_by_role(long ptr, java.lang.foreign.MemorySegment role) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_get_by_role.invokeExact(ptr, role);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_get_by_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_get_by_text = findDowncallHandle("uniffi_xcelerate_fn_method_page_get_by_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_get_by_text(long ptr, java.lang.foreign.MemorySegment text) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_get_by_text.invokeExact(ptr, text);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_get_default_timeout
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_get_default_timeout = findDowncallHandle("uniffi_xcelerate_fn_method_page_get_default_timeout", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_get_default_timeout(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_get_default_timeout.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_go_back
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_go_back = findDowncallHandle("uniffi_xcelerate_fn_method_page_go_back", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_go_back(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_go_back.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_go_forward
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_go_forward = findDowncallHandle("uniffi_xcelerate_fn_method_page_go_forward", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_go_forward(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_go_forward.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_handle_js_dialog
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_handle_js_dialog = findDowncallHandle("uniffi_xcelerate_fn_method_page_handle_js_dialog", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_BYTE, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_handle_js_dialog(long ptr, byte accept, java.lang.foreign.MemorySegment promptText) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_handle_js_dialog.invokeExact(ptr, accept, promptText);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_inject_file
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_inject_file = findDowncallHandle("uniffi_xcelerate_fn_method_page_inject_file", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_inject_file(long ptr, java.lang.foreign.MemorySegment path) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_inject_file.invokeExact(ptr, path);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_is_drag_interception_enabled
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_is_drag_interception_enabled = findDowncallHandle("uniffi_xcelerate_fn_method_page_is_drag_interception_enabled", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_is_drag_interception_enabled(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_is_drag_interception_enabled.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_keyboard_down
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_keyboard_down = findDowncallHandle("uniffi_xcelerate_fn_method_page_keyboard_down", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_keyboard_down(long ptr, java.lang.foreign.MemorySegment key) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_keyboard_down.invokeExact(ptr, key);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_keyboard_press
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_keyboard_press = findDowncallHandle("uniffi_xcelerate_fn_method_page_keyboard_press", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_keyboard_press(long ptr, java.lang.foreign.MemorySegment key) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_keyboard_press.invokeExact(ptr, key);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_keyboard_type
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_keyboard_type = findDowncallHandle("uniffi_xcelerate_fn_method_page_keyboard_type", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_keyboard_type(long ptr, java.lang.foreign.MemorySegment text) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_keyboard_type.invokeExact(ptr, text);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_keyboard_up
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_keyboard_up = findDowncallHandle("uniffi_xcelerate_fn_method_page_keyboard_up", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_keyboard_up(long ptr, java.lang.foreign.MemorySegment key) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_keyboard_up.invokeExact(ptr, key);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_listens_to
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_listens_to = findDowncallHandle("uniffi_xcelerate_fn_method_page_listens_to", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_listens_to(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_listens_to.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_main_frame
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_main_frame = findDowncallHandle("uniffi_xcelerate_fn_method_page_main_frame", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_main_frame(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_main_frame.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_metrics
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_metrics = findDowncallHandle("uniffi_xcelerate_fn_method_page_metrics", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_metrics(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_metrics.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_mouse_down
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_mouse_down = findDowncallHandle("uniffi_xcelerate_fn_method_page_mouse_down", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_mouse_down(long ptr, java.lang.foreign.MemorySegment button) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_mouse_down.invokeExact(ptr, button);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_mouse_up
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_mouse_up = findDowncallHandle("uniffi_xcelerate_fn_method_page_mouse_up", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_mouse_up(long ptr, java.lang.foreign.MemorySegment button) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_mouse_up.invokeExact(ptr, button);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_move_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_move_mouse = findDowncallHandle("uniffi_xcelerate_fn_method_page_move_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_DOUBLE, java.lang.foreign.ValueLayout.JAVA_DOUBLE));

    static long uniffi_xcelerate_fn_method_page_move_mouse(long ptr, double x, double y) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_move_mouse.invokeExact(ptr, x, y);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_navigate
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_navigate = findDowncallHandle("uniffi_xcelerate_fn_method_page_navigate", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_navigate(long ptr, java.lang.foreign.MemorySegment url) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_navigate.invokeExact(ptr, url);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_on
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_on = findDowncallHandle("uniffi_xcelerate_fn_method_page_on", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_on(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_on.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_once
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_once = findDowncallHandle("uniffi_xcelerate_fn_method_page_once", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_once(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_once.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_pdf
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_pdf = findDowncallHandle("uniffi_xcelerate_fn_method_page_pdf", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_pdf(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_pdf.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_press
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_press = findDowncallHandle("uniffi_xcelerate_fn_method_page_press", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_press(long ptr, java.lang.foreign.MemorySegment selector, java.lang.foreign.MemorySegment key) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_press.invokeExact(ptr, selector, key);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_query_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_query_selector_all = findDowncallHandle("uniffi_xcelerate_fn_method_page_query_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_query_selector_all(long ptr, java.lang.foreign.MemorySegment selector) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_query_selector_all.invokeExact(ptr, selector);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_query_selector_xpath
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_query_selector_xpath = findDowncallHandle("uniffi_xcelerate_fn_method_page_query_selector_xpath", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_query_selector_xpath(long ptr, java.lang.foreign.MemorySegment xpath) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_query_selector_xpath.invokeExact(ptr, xpath);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_raw_window_bounds
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_raw_window_bounds = findDowncallHandle("uniffi_xcelerate_fn_method_page_raw_window_bounds", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_raw_window_bounds(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_raw_window_bounds.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_reload
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_reload = findDowncallHandle("uniffi_xcelerate_fn_method_page_reload", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_reload(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_reload.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_remove_all_listeners
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_remove_all_listeners = findDowncallHandle("uniffi_xcelerate_fn_method_page_remove_all_listeners", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_remove_all_listeners(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_remove_all_listeners.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_remove_listener
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_remove_listener = findDowncallHandle("uniffi_xcelerate_fn_method_page_remove_listener", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_remove_listener(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_remove_listener.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_remove_script
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_remove_script = findDowncallHandle("uniffi_xcelerate_fn_method_page_remove_script", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_remove_script(long ptr, java.lang.foreign.MemorySegment identifier) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_remove_script.invokeExact(ptr, identifier);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_request
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_request = findDowncallHandle("uniffi_xcelerate_fn_method_page_request", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_request(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_request.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_requests
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_requests = findDowncallHandle("uniffi_xcelerate_fn_method_page_requests", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_requests(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_requests.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_route
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_route = findDowncallHandle("uniffi_xcelerate_fn_method_page_route", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_route(long ptr, java.lang.foreign.MemorySegment pattern, java.lang.foreign.MemorySegment action, java.lang.foreign.MemorySegment body, java.lang.foreign.MemorySegment contentType) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_route.invokeExact(ptr, pattern, action, body, contentType);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_route_abort
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_route_abort = findDowncallHandle("uniffi_xcelerate_fn_method_page_route_abort", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_route_abort(long ptr, java.lang.foreign.MemorySegment pattern) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_route_abort.invokeExact(ptr, pattern);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_route_from_har
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_route_from_har = findDowncallHandle("uniffi_xcelerate_fn_method_page_route_from_har", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_route_from_har(long ptr, java.lang.foreign.MemorySegment path) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_route_from_har.invokeExact(ptr, path);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_route_fulfill
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_route_fulfill = findDowncallHandle("uniffi_xcelerate_fn_method_page_route_fulfill", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_route_fulfill(long ptr, java.lang.foreign.MemorySegment pattern, java.lang.foreign.MemorySegment body, java.lang.foreign.MemorySegment contentType) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_route_fulfill.invokeExact(ptr, pattern, body, contentType);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_screenshot
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_screenshot = findDowncallHandle("uniffi_xcelerate_fn_method_page_screenshot", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_screenshot(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_screenshot.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_screenshot_full
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_screenshot_full = findDowncallHandle("uniffi_xcelerate_fn_method_page_screenshot_full", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_screenshot_full(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_screenshot_full.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_select_option
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_select_option = findDowncallHandle("uniffi_xcelerate_fn_method_page_select_option", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_select_option(long ptr, java.lang.foreign.MemorySegment selector, java.lang.foreign.MemorySegment valuesJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_select_option.invokeExact(ptr, selector, valuesJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_cache_enabled
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_cache_enabled = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_cache_enabled", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_BYTE));

    static long uniffi_xcelerate_fn_method_page_set_cache_enabled(long ptr, byte enabled) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_cache_enabled.invokeExact(ptr, enabled);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_content
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_content = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_content", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_set_content(long ptr, java.lang.foreign.MemorySegment html) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_content.invokeExact(ptr, html);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_default_timeout
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_default_timeout = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_default_timeout", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_DOUBLE));

    static long uniffi_xcelerate_fn_method_page_set_default_timeout(long ptr, double milliseconds) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_default_timeout.invokeExact(ptr, milliseconds);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_drag_interception
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_drag_interception = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_drag_interception", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_BYTE));

    static long uniffi_xcelerate_fn_method_page_set_drag_interception(long ptr, byte enabled) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_drag_interception.invokeExact(ptr, enabled);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_emulated_media_features
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_emulated_media_features = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_emulated_media_features", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_set_emulated_media_features(long ptr, java.lang.foreign.MemorySegment featuresJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_emulated_media_features.invokeExact(ptr, featuresJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_extra_http_headers
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_extra_http_headers = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_extra_http_headers", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_set_extra_http_headers(long ptr, java.lang.foreign.MemorySegment headersJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_extra_http_headers.invokeExact(ptr, headersJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_input_files
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_input_files = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_input_files", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_set_input_files(long ptr, java.lang.foreign.MemorySegment selector, java.lang.foreign.MemorySegment filesJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_input_files.invokeExact(ptr, selector, filesJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_javascript_enabled
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_javascript_enabled = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_javascript_enabled", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_BYTE));

    static long uniffi_xcelerate_fn_method_page_set_javascript_enabled(long ptr, byte enabled) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_javascript_enabled.invokeExact(ptr, enabled);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_offline
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_offline = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_offline", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_BYTE));

    static long uniffi_xcelerate_fn_method_page_set_offline(long ptr, byte offline) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_offline.invokeExact(ptr, offline);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_request_interception
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_request_interception = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_request_interception", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_BYTE));

    static long uniffi_xcelerate_fn_method_page_set_request_interception(long ptr, byte enabled) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_request_interception.invokeExact(ptr, enabled);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_storage_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_storage_state = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_storage_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_set_storage_state(long ptr, java.lang.foreign.MemorySegment stateJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_storage_state.invokeExact(ptr, stateJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_user_agent
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_user_agent = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_user_agent", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_set_user_agent(long ptr, java.lang.foreign.MemorySegment userAgent, java.lang.foreign.MemorySegment acceptLanguage) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_user_agent.invokeExact(ptr, userAgent, acceptLanguage);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_viewport_size
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_viewport_size = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_viewport_size", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_set_viewport_size(long ptr, long width, long height) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_viewport_size.invokeExact(ptr, width, height);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_window_bounds
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_window_bounds = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_window_bounds", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_set_window_bounds(long ptr, long left, long top, long width, long height) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_window_bounds.invokeExact(ptr, left, top, width, height);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_window_position
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_window_position = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_window_position", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_set_window_position(long ptr, long x, long y) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_window_position.invokeExact(ptr, x, y);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_window_size
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_window_size = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_window_size", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_set_window_size(long ptr, long width, long height) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_window_size.invokeExact(ptr, width, height);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_set_window_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_set_window_state = findDowncallHandle("uniffi_xcelerate_fn_method_page_set_window_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_set_window_state(long ptr, java.lang.foreign.MemorySegment state) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_set_window_state.invokeExact(ptr, state);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_start_screencast
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_start_screencast = findDowncallHandle("uniffi_xcelerate_fn_method_page_start_screencast", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_start_screencast(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_start_screencast.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_start_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_start_tracing = findDowncallHandle("uniffi_xcelerate_fn_method_page_start_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_start_tracing(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_start_tracing.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_stop_screencast
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_stop_screencast = findDowncallHandle("uniffi_xcelerate_fn_method_page_stop_screencast", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_stop_screencast(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_stop_screencast.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_stop_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_stop_tracing = findDowncallHandle("uniffi_xcelerate_fn_method_page_stop_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_stop_tracing(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_stop_tracing.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_storage_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_storage_state = findDowncallHandle("uniffi_xcelerate_fn_method_page_storage_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_storage_state(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_storage_state.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_target_id
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_target_id = findDowncallHandle("uniffi_xcelerate_fn_method_page_target_id", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_page_target_id(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_page_target_id.invokeExact(_allocator, ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_title
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_title = findDowncallHandle("uniffi_xcelerate_fn_method_page_title", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_title(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_title.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_touch_tap
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_touch_tap = findDowncallHandle("uniffi_xcelerate_fn_method_page_touch_tap", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_DOUBLE, java.lang.foreign.ValueLayout.JAVA_DOUBLE));

    static long uniffi_xcelerate_fn_method_page_touch_tap(long ptr, double x, double y) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_touch_tap.invokeExact(ptr, x, y);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_unroute
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_unroute = findDowncallHandle("uniffi_xcelerate_fn_method_page_unroute", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_unroute(long ptr, java.lang.foreign.MemorySegment pattern) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_unroute.invokeExact(ptr, pattern);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_unroute_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_unroute_all = findDowncallHandle("uniffi_xcelerate_fn_method_page_unroute_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_unroute_all(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_unroute_all.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_url
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_url = findDowncallHandle("uniffi_xcelerate_fn_method_page_url", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_url(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_url.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_wait_for_event
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_wait_for_event = findDowncallHandle("uniffi_xcelerate_fn_method_page_wait_for_event", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_wait_for_event(long ptr, java.lang.foreign.MemorySegment eventName, long timeoutMs) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_wait_for_event.invokeExact(ptr, eventName, timeoutMs);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_wait_for_event_default
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_wait_for_event_default = findDowncallHandle("uniffi_xcelerate_fn_method_page_wait_for_event_default", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_wait_for_event_default(long ptr, java.lang.foreign.MemorySegment eventName) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_wait_for_event_default.invokeExact(ptr, eventName);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_wait_for_function
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_wait_for_function = findDowncallHandle("uniffi_xcelerate_fn_method_page_wait_for_function", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_wait_for_function(long ptr, java.lang.foreign.MemorySegment expression, long timeoutMs) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_wait_for_function.invokeExact(ptr, expression, timeoutMs);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_wait_for_navigation
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_wait_for_navigation = findDowncallHandle("uniffi_xcelerate_fn_method_page_wait_for_navigation", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_wait_for_navigation(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_wait_for_navigation.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_wait_for_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_wait_for_selector = findDowncallHandle("uniffi_xcelerate_fn_method_page_wait_for_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_page_wait_for_selector(long ptr, java.lang.foreign.MemorySegment selector) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_wait_for_selector.invokeExact(ptr, selector);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_wait_for_xpath
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_wait_for_xpath = findDowncallHandle("uniffi_xcelerate_fn_method_page_wait_for_xpath", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_wait_for_xpath(long ptr, java.lang.foreign.MemorySegment xpath, long timeoutMs) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_wait_for_xpath.invokeExact(ptr, xpath, timeoutMs);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_window_id
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_window_id = findDowncallHandle("uniffi_xcelerate_fn_method_page_window_id", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_window_id(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_window_id.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_window_position
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_window_position = findDowncallHandle("uniffi_xcelerate_fn_method_page_window_position", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_window_position(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_window_position.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_window_rect
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_window_rect = findDowncallHandle("uniffi_xcelerate_fn_method_page_window_rect", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_window_rect(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_window_rect.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_page_window_size
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_page_window_size = findDowncallHandle("uniffi_xcelerate_fn_method_page_window_size", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG));

    static long uniffi_xcelerate_fn_method_page_window_size(long ptr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_page_window_size.invokeExact(ptr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_clone_pluginhandle
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_clone_pluginhandle = findDowncallHandle("uniffi_xcelerate_fn_clone_pluginhandle", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static long uniffi_xcelerate_fn_clone_pluginhandle(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_uniffi_xcelerate_fn_clone_pluginhandle.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_free_pluginhandle
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_free_pluginhandle = findDowncallHandle("uniffi_xcelerate_fn_free_pluginhandle", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static void uniffi_xcelerate_fn_free_pluginhandle(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            MH_uniffi_xcelerate_fn_free_pluginhandle.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_pluginhandle_invoke
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_pluginhandle_invoke = findDowncallHandle("uniffi_xcelerate_fn_method_pluginhandle_invoke", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, RustBuffer.LAYOUT, RustBuffer.LAYOUT));

    static long uniffi_xcelerate_fn_method_pluginhandle_invoke(long ptr, java.lang.foreign.MemorySegment op, java.lang.foreign.MemorySegment argsJson) {
        try {
            return (long) MH_uniffi_xcelerate_fn_method_pluginhandle_invoke.invokeExact(ptr, op, argsJson);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_pluginhandle_ops
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_pluginhandle_ops = findDowncallHandle("uniffi_xcelerate_fn_method_pluginhandle_ops", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_pluginhandle_ops(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_pluginhandle_ops.invokeExact(_allocator, ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_fn_method_pluginhandle_plugin_name
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_fn_method_pluginhandle_plugin_name = findDowncallHandle("uniffi_xcelerate_fn_method_pluginhandle_plugin_name", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment uniffi_xcelerate_fn_method_pluginhandle_plugin_name(java.lang.foreign.SegmentAllocator _allocator, long ptr, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_uniffi_xcelerate_fn_method_pluginhandle_plugin_name.invokeExact(_allocator, ptr, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rustbuffer_alloc
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rustbuffer_alloc = findDowncallHandle("ffi_xcelerate_rustbuffer_alloc", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment ffi_xcelerate_rustbuffer_alloc(java.lang.foreign.SegmentAllocator _allocator, long size, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_ffi_xcelerate_rustbuffer_alloc.invokeExact(_allocator, size, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rustbuffer_from_bytes
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rustbuffer_from_bytes = findDowncallHandle("ffi_xcelerate_rustbuffer_from_bytes", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, ForeignBytes.LAYOUT, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment ffi_xcelerate_rustbuffer_from_bytes(java.lang.foreign.SegmentAllocator _allocator, java.lang.foreign.MemorySegment bytes, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_ffi_xcelerate_rustbuffer_from_bytes.invokeExact(_allocator, bytes, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rustbuffer_free
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rustbuffer_free = findDowncallHandle("ffi_xcelerate_rustbuffer_free", java.lang.foreign.FunctionDescriptor.ofVoid(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.ADDRESS));

    static void ffi_xcelerate_rustbuffer_free(java.lang.foreign.MemorySegment buf, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            MH_ffi_xcelerate_rustbuffer_free.invokeExact(buf, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rustbuffer_reserve
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rustbuffer_reserve = findDowncallHandle("ffi_xcelerate_rustbuffer_reserve", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment ffi_xcelerate_rustbuffer_reserve(java.lang.foreign.SegmentAllocator _allocator, java.lang.foreign.MemorySegment buf, long additional, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_ffi_xcelerate_rustbuffer_reserve.invokeExact(_allocator, buf, additional, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_u8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_u8 = findDowncallHandle("ffi_xcelerate_rust_future_poll_u8", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_u8(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_u8.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_u8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_u8 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_u8", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_u8(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_u8.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_u8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_u8 = findDowncallHandle("ffi_xcelerate_rust_future_free_u8", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_u8(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_u8.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_u8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_u8 = findDowncallHandle("ffi_xcelerate_rust_future_complete_u8", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_BYTE, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static byte ffi_xcelerate_rust_future_complete_u8(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (byte) MH_ffi_xcelerate_rust_future_complete_u8.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_i8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_i8 = findDowncallHandle("ffi_xcelerate_rust_future_poll_i8", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_i8(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_i8.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_i8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_i8 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_i8", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_i8(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_i8.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_i8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_i8 = findDowncallHandle("ffi_xcelerate_rust_future_free_i8", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_i8(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_i8.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_i8
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_i8 = findDowncallHandle("ffi_xcelerate_rust_future_complete_i8", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_BYTE, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static byte ffi_xcelerate_rust_future_complete_i8(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (byte) MH_ffi_xcelerate_rust_future_complete_i8.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_u16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_u16 = findDowncallHandle("ffi_xcelerate_rust_future_poll_u16", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_u16(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_u16.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_u16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_u16 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_u16", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_u16(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_u16.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_u16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_u16 = findDowncallHandle("ffi_xcelerate_rust_future_free_u16", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_u16(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_u16.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_u16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_u16 = findDowncallHandle("ffi_xcelerate_rust_future_complete_u16", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static short ffi_xcelerate_rust_future_complete_u16(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (short) MH_ffi_xcelerate_rust_future_complete_u16.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_i16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_i16 = findDowncallHandle("ffi_xcelerate_rust_future_poll_i16", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_i16(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_i16.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_i16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_i16 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_i16", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_i16(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_i16.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_i16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_i16 = findDowncallHandle("ffi_xcelerate_rust_future_free_i16", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_i16(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_i16.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_i16
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_i16 = findDowncallHandle("ffi_xcelerate_rust_future_complete_i16", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static short ffi_xcelerate_rust_future_complete_i16(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (short) MH_ffi_xcelerate_rust_future_complete_i16.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_u32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_u32 = findDowncallHandle("ffi_xcelerate_rust_future_poll_u32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_u32(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_u32.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_u32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_u32 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_u32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_u32(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_u32.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_u32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_u32 = findDowncallHandle("ffi_xcelerate_rust_future_free_u32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_u32(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_u32.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_u32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_u32 = findDowncallHandle("ffi_xcelerate_rust_future_complete_u32", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_INT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static int ffi_xcelerate_rust_future_complete_u32(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (int) MH_ffi_xcelerate_rust_future_complete_u32.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_i32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_i32 = findDowncallHandle("ffi_xcelerate_rust_future_poll_i32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_i32(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_i32.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_i32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_i32 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_i32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_i32(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_i32.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_i32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_i32 = findDowncallHandle("ffi_xcelerate_rust_future_free_i32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_i32(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_i32.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_i32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_i32 = findDowncallHandle("ffi_xcelerate_rust_future_complete_i32", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_INT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static int ffi_xcelerate_rust_future_complete_i32(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (int) MH_ffi_xcelerate_rust_future_complete_i32.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_u64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_u64 = findDowncallHandle("ffi_xcelerate_rust_future_poll_u64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_u64(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_u64.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_u64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_u64 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_u64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_u64(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_u64.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_u64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_u64 = findDowncallHandle("ffi_xcelerate_rust_future_free_u64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_u64(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_u64.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_u64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_u64 = findDowncallHandle("ffi_xcelerate_rust_future_complete_u64", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static long ffi_xcelerate_rust_future_complete_u64(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_ffi_xcelerate_rust_future_complete_u64.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_i64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_i64 = findDowncallHandle("ffi_xcelerate_rust_future_poll_i64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_i64(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_i64.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_i64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_i64 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_i64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_i64(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_i64.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_i64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_i64 = findDowncallHandle("ffi_xcelerate_rust_future_free_i64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_i64(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_i64.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_i64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_i64 = findDowncallHandle("ffi_xcelerate_rust_future_complete_i64", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static long ffi_xcelerate_rust_future_complete_i64(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (long) MH_ffi_xcelerate_rust_future_complete_i64.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_f32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_f32 = findDowncallHandle("ffi_xcelerate_rust_future_poll_f32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_f32(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_f32.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_f32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_f32 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_f32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_f32(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_f32.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_f32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_f32 = findDowncallHandle("ffi_xcelerate_rust_future_free_f32", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_f32(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_f32.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_f32
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_f32 = findDowncallHandle("ffi_xcelerate_rust_future_complete_f32", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_FLOAT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static float ffi_xcelerate_rust_future_complete_f32(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (float) MH_ffi_xcelerate_rust_future_complete_f32.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_f64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_f64 = findDowncallHandle("ffi_xcelerate_rust_future_poll_f64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_f64(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_f64.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_f64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_f64 = findDowncallHandle("ffi_xcelerate_rust_future_cancel_f64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_f64(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_f64.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_f64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_f64 = findDowncallHandle("ffi_xcelerate_rust_future_free_f64", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_f64(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_f64.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_f64
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_f64 = findDowncallHandle("ffi_xcelerate_rust_future_complete_f64", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_DOUBLE, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static double ffi_xcelerate_rust_future_complete_f64(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (double) MH_ffi_xcelerate_rust_future_complete_f64.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_rust_buffer
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_rust_buffer = findDowncallHandle("ffi_xcelerate_rust_future_poll_rust_buffer", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_rust_buffer(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_rust_buffer.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_rust_buffer
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_rust_buffer = findDowncallHandle("ffi_xcelerate_rust_future_cancel_rust_buffer", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_rust_buffer(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_rust_buffer.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_rust_buffer
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_rust_buffer = findDowncallHandle("ffi_xcelerate_rust_future_free_rust_buffer", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_rust_buffer(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_rust_buffer.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_rust_buffer
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_rust_buffer = findDowncallHandle("ffi_xcelerate_rust_future_complete_rust_buffer", java.lang.foreign.FunctionDescriptor.of(RustBuffer.LAYOUT, java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static java.lang.foreign.MemorySegment ffi_xcelerate_rust_future_complete_rust_buffer(java.lang.foreign.SegmentAllocator _allocator, long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            return (java.lang.foreign.MemorySegment) MH_ffi_xcelerate_rust_future_complete_rust_buffer.invokeExact(_allocator, handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_poll_void
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_poll_void = findDowncallHandle("ffi_xcelerate_rust_future_poll_void", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS, java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_poll_void(long handle, java.lang.foreign.MemorySegment callback, long callbackData) {
        try {
            MH_ffi_xcelerate_rust_future_poll_void.invokeExact(handle, callback, callbackData);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_cancel_void
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_cancel_void = findDowncallHandle("ffi_xcelerate_rust_future_cancel_void", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_cancel_void(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_cancel_void.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_free_void
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_free_void = findDowncallHandle("ffi_xcelerate_rust_future_free_void", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG));

    static void ffi_xcelerate_rust_future_free_void(long handle) {
        try {
            MH_ffi_xcelerate_rust_future_free_void.invokeExact(handle);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_rust_future_complete_void
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_rust_future_complete_void = findDowncallHandle("ffi_xcelerate_rust_future_complete_void", java.lang.foreign.FunctionDescriptor.ofVoid(java.lang.foreign.ValueLayout.JAVA_LONG, java.lang.foreign.ValueLayout.ADDRESS));

    static void ffi_xcelerate_rust_future_complete_void(long handle, java.lang.foreign.MemorySegment uniffiOutErr) {
        try {
            MH_ffi_xcelerate_rust_future_complete_void.invokeExact(handle, uniffiOutErr);
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_audit_log
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_audit_log = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_audit_log", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_audit_log() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_audit_log.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_audit_verify
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_audit_verify = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_audit_verify", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_audit_verify() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_audit_verify.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_available_plugins
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_available_plugins = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_available_plugins", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_available_plugins() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_available_plugins.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_browser_contexts
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_browser_contexts = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_browser_contexts", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_browser_contexts() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_browser_contexts.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_capabilities
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_capabilities = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_capabilities", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_capabilities() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_capabilities.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_close
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_close = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_close", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_close() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_close.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_cookies
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_cookies = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_cookies", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_cookies() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_cookies.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_delete_cookie
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_delete_cookie = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_delete_cookie", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_delete_cookie() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_delete_cookie.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_event_names
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_event_names = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_event_names", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_event_names() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_event_names.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_grant_permissions
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_grant_permissions = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_grant_permissions", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_grant_permissions() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_grant_permissions.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_is_connected
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_is_connected = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_is_connected", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_is_connected() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_is_connected.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_listens_to
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_listens_to = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_listens_to", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_listens_to() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_listens_to.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_load_plugin
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_load_plugin = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_load_plugin", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_load_plugin() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_load_plugin.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_new_context
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_new_context = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_new_context", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_new_context() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_new_context.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_new_page
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_new_page = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_new_page", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_new_page() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_new_page.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_on
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_on = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_on", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_on() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_on.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_once
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_once = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_once", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_once() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_once.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_plugin
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_plugin = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_plugin", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_plugin() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_plugin.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_plugin_names
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_plugin_names = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_plugin_names", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_plugin_names() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_plugin_names.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_remove_all_listeners
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_remove_all_listeners = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_remove_all_listeners", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_remove_all_listeners() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_remove_all_listeners.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_remove_listener
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_remove_listener = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_remove_listener", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_remove_listener() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_remove_listener.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_reset_permissions
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_reset_permissions = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_reset_permissions", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_reset_permissions() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_reset_permissions.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_set_cookie
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_set_cookie = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_set_cookie", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_set_cookie() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_set_cookie.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_set_download_behavior
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_set_download_behavior = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_set_download_behavior", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_set_download_behavior() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_set_download_behavior.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_start_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_start_tracing = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_start_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_start_tracing() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_start_tracing.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_stop_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_stop_tracing = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_stop_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_stop_tracing() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_stop_tracing.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_targets
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_targets = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_targets", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_targets() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_targets.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_use_plugin
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_use_plugin = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_use_plugin", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_use_plugin() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_use_plugin.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_user_agent
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_user_agent = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_user_agent", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_user_agent() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_user_agent.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_version
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_version = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_version", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_version() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_version.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_wait_for_event
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_wait_for_event = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_wait_for_event", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_wait_for_event() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_wait_for_event.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_wait_for_event_default
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_wait_for_event_default = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_wait_for_event_default", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_wait_for_event_default() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_wait_for_event_default.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_browser_ws_endpoint
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_browser_ws_endpoint = findDowncallHandle("uniffi_xcelerate_checksum_method_browser_ws_endpoint", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_browser_ws_endpoint() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_browser_ws_endpoint.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_attribute
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_attribute = findDowncallHandle("uniffi_xcelerate_checksum_method_element_attribute", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_attribute() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_attribute.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_call_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_call_bool = findDowncallHandle("uniffi_xcelerate_checksum_method_element_call_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_call_bool() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_call_bool.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_call_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_call_json = findDowncallHandle("uniffi_xcelerate_checksum_method_element_call_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_call_json() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_call_json.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_call_on_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_call_on_selector = findDowncallHandle("uniffi_xcelerate_checksum_method_element_call_on_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_call_on_selector() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_call_on_selector.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_call_on_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_call_on_selector_all = findDowncallHandle("uniffi_xcelerate_checksum_method_element_call_on_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_call_on_selector_all() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_call_on_selector_all.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_call_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_call_string = findDowncallHandle("uniffi_xcelerate_checksum_method_element_call_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_call_string() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_call_string.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_click
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_click = findDowncallHandle("uniffi_xcelerate_checksum_method_element_click", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_click() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_click.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_click_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_click_mouse = findDowncallHandle("uniffi_xcelerate_checksum_method_element_click_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_click_mouse() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_click_mouse.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_count
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_count = findDowncallHandle("uniffi_xcelerate_checksum_method_element_count", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_count() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_count.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_dispose
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_dispose = findDowncallHandle("uniffi_xcelerate_checksum_method_element_dispose", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_dispose() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_dispose.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_evaluate_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_evaluate_bool = findDowncallHandle("uniffi_xcelerate_checksum_method_element_evaluate_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_evaluate_bool() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_evaluate_bool.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_evaluate_handle
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_evaluate_handle = findDowncallHandle("uniffi_xcelerate_checksum_method_element_evaluate_handle", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_evaluate_handle() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_evaluate_handle.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_evaluate_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_evaluate_json = findDowncallHandle("uniffi_xcelerate_checksum_method_element_evaluate_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_evaluate_json() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_evaluate_json.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_evaluate_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_evaluate_string = findDowncallHandle("uniffi_xcelerate_checksum_method_element_evaluate_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_evaluate_string() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_evaluate_string.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_focus
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_focus = findDowncallHandle("uniffi_xcelerate_checksum_method_element_focus", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_focus() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_focus.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_get_by_label
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_get_by_label = findDowncallHandle("uniffi_xcelerate_checksum_method_element_get_by_label", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_get_by_label() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_get_by_label.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_get_by_role
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_get_by_role = findDowncallHandle("uniffi_xcelerate_checksum_method_element_get_by_role", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_get_by_role() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_get_by_role.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_get_by_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_get_by_text = findDowncallHandle("uniffi_xcelerate_checksum_method_element_get_by_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_get_by_text() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_get_by_text.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_get_properties
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_get_properties = findDowncallHandle("uniffi_xcelerate_checksum_method_element_get_properties", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_get_properties() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_get_properties.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_hover
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_hover = findDowncallHandle("uniffi_xcelerate_checksum_method_element_hover", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_hover() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_hover.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_hover_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_hover_mouse = findDowncallHandle("uniffi_xcelerate_checksum_method_element_hover_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_hover_mouse() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_hover_mouse.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_inner_html
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_inner_html = findDowncallHandle("uniffi_xcelerate_checksum_method_element_inner_html", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_inner_html() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_inner_html.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_press
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_press = findDowncallHandle("uniffi_xcelerate_checksum_method_element_press", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_press() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_press.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_query_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_query_selector = findDowncallHandle("uniffi_xcelerate_checksum_method_element_query_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_query_selector() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_query_selector.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_query_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_query_selector_all = findDowncallHandle("uniffi_xcelerate_checksum_method_element_query_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_query_selector_all() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_query_selector_all.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_query_selector_attr
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_query_selector_attr = findDowncallHandle("uniffi_xcelerate_checksum_method_element_query_selector_attr", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_query_selector_attr() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_query_selector_attr.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_query_selector_xpath
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_query_selector_xpath = findDowncallHandle("uniffi_xcelerate_checksum_method_element_query_selector_xpath", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_query_selector_xpath() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_query_selector_xpath.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_screenshot
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_screenshot = findDowncallHandle("uniffi_xcelerate_checksum_method_element_screenshot", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_screenshot() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_screenshot.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_screenshot_base64
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_screenshot_base64 = findDowncallHandle("uniffi_xcelerate_checksum_method_element_screenshot_base64", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_screenshot_base64() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_screenshot_base64.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_select_option
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_select_option = findDowncallHandle("uniffi_xcelerate_checksum_method_element_select_option", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_select_option() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_select_option.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_set_input_files
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_set_input_files = findDowncallHandle("uniffi_xcelerate_checksum_method_element_set_input_files", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_set_input_files() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_set_input_files.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_text = findDowncallHandle("uniffi_xcelerate_checksum_method_element_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_text() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_text.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_type_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_type_text = findDowncallHandle("uniffi_xcelerate_checksum_method_element_type_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_type_text() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_type_text.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_element_wait_for_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_element_wait_for_selector = findDowncallHandle("uniffi_xcelerate_checksum_method_element_wait_for_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_element_wait_for_selector() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_element_wait_for_selector.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_activate
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_activate = findDowncallHandle("uniffi_xcelerate_checksum_method_page_activate", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_activate() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_activate.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_activate_target
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_activate_target = findDowncallHandle("uniffi_xcelerate_checksum_method_page_activate_target", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_activate_target() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_activate_target.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document = findDowncallHandle("uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_add_script_to_evaluate_on_new_document.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_add_style_tag
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_add_style_tag = findDowncallHandle("uniffi_xcelerate_checksum_method_page_add_style_tag", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_add_style_tag() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_add_style_tag.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_authenticate
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_authenticate = findDowncallHandle("uniffi_xcelerate_checksum_method_page_authenticate", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_authenticate() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_authenticate.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_bring_to_front
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_bring_to_front = findDowncallHandle("uniffi_xcelerate_checksum_method_page_bring_to_front", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_bring_to_front() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_bring_to_front.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_call_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_call_bool = findDowncallHandle("uniffi_xcelerate_checksum_method_page_call_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_call_bool() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_call_bool.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_call_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_call_json = findDowncallHandle("uniffi_xcelerate_checksum_method_page_call_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_call_json() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_call_json.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_call_on_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_call_on_selector = findDowncallHandle("uniffi_xcelerate_checksum_method_page_call_on_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_call_on_selector() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_call_on_selector.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_call_on_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_call_on_selector_all = findDowncallHandle("uniffi_xcelerate_checksum_method_page_call_on_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_call_on_selector_all() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_call_on_selector_all.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_call_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_call_string = findDowncallHandle("uniffi_xcelerate_checksum_method_page_call_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_call_string() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_call_string.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_clear_requests
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_clear_requests = findDowncallHandle("uniffi_xcelerate_checksum_method_page_clear_requests", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_clear_requests() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_clear_requests.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_click_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_click_mouse = findDowncallHandle("uniffi_xcelerate_checksum_method_page_click_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_click_mouse() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_click_mouse.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_close
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_close = findDowncallHandle("uniffi_xcelerate_checksum_method_page_close", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_close() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_close.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_content
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_content = findDowncallHandle("uniffi_xcelerate_checksum_method_page_content", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_content() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_content.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_cookie
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_cookie = findDowncallHandle("uniffi_xcelerate_checksum_method_page_cookie", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_cookie() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_cookie.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_cookies
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_cookies = findDowncallHandle("uniffi_xcelerate_checksum_method_page_cookies", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_cookies() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_cookies.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_coverage_start_css
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_coverage_start_css = findDowncallHandle("uniffi_xcelerate_checksum_method_page_coverage_start_css", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_coverage_start_css() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_coverage_start_css.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_coverage_start_js
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_coverage_start_js = findDowncallHandle("uniffi_xcelerate_checksum_method_page_coverage_start_js", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_coverage_start_js() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_coverage_start_js.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_coverage_stop_css
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_coverage_stop_css = findDowncallHandle("uniffi_xcelerate_checksum_method_page_coverage_stop_css", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_coverage_stop_css() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_coverage_stop_css.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_coverage_stop_js
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_coverage_stop_js = findDowncallHandle("uniffi_xcelerate_checksum_method_page_coverage_stop_js", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_coverage_stop_js() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_coverage_stop_js.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_create_pdf_stream
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_create_pdf_stream = findDowncallHandle("uniffi_xcelerate_checksum_method_page_create_pdf_stream", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_create_pdf_stream() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_create_pdf_stream.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_decode_base64
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_decode_base64 = findDowncallHandle("uniffi_xcelerate_checksum_method_page_decode_base64", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_decode_base64() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_decode_base64.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_default_timeout
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_default_timeout = findDowncallHandle("uniffi_xcelerate_checksum_method_page_default_timeout", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_default_timeout() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_default_timeout.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_document_element
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_document_element = findDowncallHandle("uniffi_xcelerate_checksum_method_page_document_element", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_document_element() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_document_element.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_emulate_idle_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_emulate_idle_state = findDowncallHandle("uniffi_xcelerate_checksum_method_page_emulate_idle_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_emulate_idle_state() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_emulate_idle_state.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_emulate_media
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_emulate_media = findDowncallHandle("uniffi_xcelerate_checksum_method_page_emulate_media", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_emulate_media() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_emulate_media.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_ensure_interception
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_ensure_interception = findDowncallHandle("uniffi_xcelerate_checksum_method_page_ensure_interception", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_ensure_interception() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_ensure_interception.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_evaluate_bool
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_evaluate_bool = findDowncallHandle("uniffi_xcelerate_checksum_method_page_evaluate_bool", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_evaluate_bool() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_evaluate_bool.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_evaluate_handle
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_evaluate_handle = findDowncallHandle("uniffi_xcelerate_checksum_method_page_evaluate_handle", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_evaluate_handle() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_evaluate_handle.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_evaluate_json
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_evaluate_json = findDowncallHandle("uniffi_xcelerate_checksum_method_page_evaluate_json", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_evaluate_json() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_evaluate_json.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_evaluate_string
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_evaluate_string = findDowncallHandle("uniffi_xcelerate_checksum_method_page_evaluate_string", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_evaluate_string() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_evaluate_string.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_event_names
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_event_names = findDowncallHandle("uniffi_xcelerate_checksum_method_page_event_names", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_event_names() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_event_names.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_execute_cdp_cmd
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_execute_cdp_cmd = findDowncallHandle("uniffi_xcelerate_checksum_method_page_execute_cdp_cmd", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_execute_cdp_cmd() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_execute_cdp_cmd.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_find_element
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_find_element = findDowncallHandle("uniffi_xcelerate_checksum_method_page_find_element", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_find_element() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_find_element.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_frame
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_frame = findDowncallHandle("uniffi_xcelerate_checksum_method_page_frame", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_frame() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_frame.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_frame_name
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_frame_name = findDowncallHandle("uniffi_xcelerate_checksum_method_page_frame_name", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_frame_name() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_frame_name.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_frames
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_frames = findDowncallHandle("uniffi_xcelerate_checksum_method_page_frames", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_frames() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_frames.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_get_by_label
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_get_by_label = findDowncallHandle("uniffi_xcelerate_checksum_method_page_get_by_label", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_get_by_label() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_get_by_label.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_get_by_role
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_get_by_role = findDowncallHandle("uniffi_xcelerate_checksum_method_page_get_by_role", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_get_by_role() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_get_by_role.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_get_by_text
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_get_by_text = findDowncallHandle("uniffi_xcelerate_checksum_method_page_get_by_text", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_get_by_text() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_get_by_text.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_get_default_timeout
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_get_default_timeout = findDowncallHandle("uniffi_xcelerate_checksum_method_page_get_default_timeout", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_get_default_timeout() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_get_default_timeout.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_go_back
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_go_back = findDowncallHandle("uniffi_xcelerate_checksum_method_page_go_back", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_go_back() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_go_back.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_go_forward
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_go_forward = findDowncallHandle("uniffi_xcelerate_checksum_method_page_go_forward", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_go_forward() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_go_forward.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_handle_js_dialog
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_handle_js_dialog = findDowncallHandle("uniffi_xcelerate_checksum_method_page_handle_js_dialog", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_handle_js_dialog() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_handle_js_dialog.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_inject_file
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_inject_file = findDowncallHandle("uniffi_xcelerate_checksum_method_page_inject_file", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_inject_file() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_inject_file.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled = findDowncallHandle("uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_is_drag_interception_enabled.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_keyboard_down
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_keyboard_down = findDowncallHandle("uniffi_xcelerate_checksum_method_page_keyboard_down", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_keyboard_down() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_keyboard_down.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_keyboard_press
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_keyboard_press = findDowncallHandle("uniffi_xcelerate_checksum_method_page_keyboard_press", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_keyboard_press() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_keyboard_press.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_keyboard_type
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_keyboard_type = findDowncallHandle("uniffi_xcelerate_checksum_method_page_keyboard_type", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_keyboard_type() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_keyboard_type.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_keyboard_up
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_keyboard_up = findDowncallHandle("uniffi_xcelerate_checksum_method_page_keyboard_up", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_keyboard_up() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_keyboard_up.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_listens_to
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_listens_to = findDowncallHandle("uniffi_xcelerate_checksum_method_page_listens_to", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_listens_to() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_listens_to.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_main_frame
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_main_frame = findDowncallHandle("uniffi_xcelerate_checksum_method_page_main_frame", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_main_frame() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_main_frame.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_metrics
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_metrics = findDowncallHandle("uniffi_xcelerate_checksum_method_page_metrics", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_metrics() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_metrics.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_mouse_down
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_mouse_down = findDowncallHandle("uniffi_xcelerate_checksum_method_page_mouse_down", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_mouse_down() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_mouse_down.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_mouse_up
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_mouse_up = findDowncallHandle("uniffi_xcelerate_checksum_method_page_mouse_up", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_mouse_up() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_mouse_up.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_move_mouse
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_move_mouse = findDowncallHandle("uniffi_xcelerate_checksum_method_page_move_mouse", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_move_mouse() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_move_mouse.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_navigate
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_navigate = findDowncallHandle("uniffi_xcelerate_checksum_method_page_navigate", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_navigate() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_navigate.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_on
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_on = findDowncallHandle("uniffi_xcelerate_checksum_method_page_on", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_on() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_on.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_once
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_once = findDowncallHandle("uniffi_xcelerate_checksum_method_page_once", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_once() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_once.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_pdf
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_pdf = findDowncallHandle("uniffi_xcelerate_checksum_method_page_pdf", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_pdf() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_pdf.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_press
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_press = findDowncallHandle("uniffi_xcelerate_checksum_method_page_press", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_press() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_press.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_query_selector_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_query_selector_all = findDowncallHandle("uniffi_xcelerate_checksum_method_page_query_selector_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_query_selector_all() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_query_selector_all.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_query_selector_xpath
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_query_selector_xpath = findDowncallHandle("uniffi_xcelerate_checksum_method_page_query_selector_xpath", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_query_selector_xpath() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_query_selector_xpath.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_raw_window_bounds
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_raw_window_bounds = findDowncallHandle("uniffi_xcelerate_checksum_method_page_raw_window_bounds", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_raw_window_bounds() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_raw_window_bounds.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_reload
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_reload = findDowncallHandle("uniffi_xcelerate_checksum_method_page_reload", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_reload() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_reload.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_remove_all_listeners
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_remove_all_listeners = findDowncallHandle("uniffi_xcelerate_checksum_method_page_remove_all_listeners", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_remove_all_listeners() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_remove_all_listeners.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_remove_listener
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_remove_listener = findDowncallHandle("uniffi_xcelerate_checksum_method_page_remove_listener", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_remove_listener() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_remove_listener.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_remove_script
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_remove_script = findDowncallHandle("uniffi_xcelerate_checksum_method_page_remove_script", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_remove_script() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_remove_script.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_request
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_request = findDowncallHandle("uniffi_xcelerate_checksum_method_page_request", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_request() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_request.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_requests
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_requests = findDowncallHandle("uniffi_xcelerate_checksum_method_page_requests", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_requests() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_requests.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_route
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_route = findDowncallHandle("uniffi_xcelerate_checksum_method_page_route", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_route() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_route.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_route_abort
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_route_abort = findDowncallHandle("uniffi_xcelerate_checksum_method_page_route_abort", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_route_abort() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_route_abort.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_route_from_har
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_route_from_har = findDowncallHandle("uniffi_xcelerate_checksum_method_page_route_from_har", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_route_from_har() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_route_from_har.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_route_fulfill
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_route_fulfill = findDowncallHandle("uniffi_xcelerate_checksum_method_page_route_fulfill", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_route_fulfill() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_route_fulfill.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_screenshot
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_screenshot = findDowncallHandle("uniffi_xcelerate_checksum_method_page_screenshot", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_screenshot() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_screenshot.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_screenshot_full
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_screenshot_full = findDowncallHandle("uniffi_xcelerate_checksum_method_page_screenshot_full", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_screenshot_full() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_screenshot_full.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_select_option
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_select_option = findDowncallHandle("uniffi_xcelerate_checksum_method_page_select_option", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_select_option() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_select_option.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_cache_enabled
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_cache_enabled = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_cache_enabled", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_cache_enabled() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_cache_enabled.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_content
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_content = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_content", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_content() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_content.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_default_timeout
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_default_timeout = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_default_timeout", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_default_timeout() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_default_timeout.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_drag_interception
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_drag_interception = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_drag_interception", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_drag_interception() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_drag_interception.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_emulated_media_features
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_emulated_media_features = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_emulated_media_features", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_emulated_media_features() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_emulated_media_features.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_extra_http_headers
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_extra_http_headers = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_extra_http_headers", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_extra_http_headers() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_extra_http_headers.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_input_files
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_input_files = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_input_files", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_input_files() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_input_files.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_javascript_enabled
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_javascript_enabled = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_javascript_enabled", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_javascript_enabled() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_javascript_enabled.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_offline
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_offline = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_offline", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_offline() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_offline.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_request_interception
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_request_interception = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_request_interception", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_request_interception() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_request_interception.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_storage_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_storage_state = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_storage_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_storage_state() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_storage_state.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_user_agent
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_user_agent = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_user_agent", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_user_agent() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_user_agent.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_viewport_size
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_viewport_size = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_viewport_size", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_viewport_size() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_viewport_size.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_window_bounds
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_window_bounds = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_window_bounds", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_window_bounds() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_window_bounds.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_window_position
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_window_position = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_window_position", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_window_position() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_window_position.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_window_size
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_window_size = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_window_size", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_window_size() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_window_size.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_set_window_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_set_window_state = findDowncallHandle("uniffi_xcelerate_checksum_method_page_set_window_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_set_window_state() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_set_window_state.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_start_screencast
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_start_screencast = findDowncallHandle("uniffi_xcelerate_checksum_method_page_start_screencast", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_start_screencast() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_start_screencast.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_start_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_start_tracing = findDowncallHandle("uniffi_xcelerate_checksum_method_page_start_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_start_tracing() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_start_tracing.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_stop_screencast
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_stop_screencast = findDowncallHandle("uniffi_xcelerate_checksum_method_page_stop_screencast", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_stop_screencast() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_stop_screencast.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_stop_tracing
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_stop_tracing = findDowncallHandle("uniffi_xcelerate_checksum_method_page_stop_tracing", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_stop_tracing() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_stop_tracing.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_storage_state
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_storage_state = findDowncallHandle("uniffi_xcelerate_checksum_method_page_storage_state", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_storage_state() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_storage_state.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_target_id
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_target_id = findDowncallHandle("uniffi_xcelerate_checksum_method_page_target_id", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_target_id() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_target_id.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_title
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_title = findDowncallHandle("uniffi_xcelerate_checksum_method_page_title", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_title() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_title.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_touch_tap
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_touch_tap = findDowncallHandle("uniffi_xcelerate_checksum_method_page_touch_tap", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_touch_tap() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_touch_tap.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_unroute
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_unroute = findDowncallHandle("uniffi_xcelerate_checksum_method_page_unroute", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_unroute() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_unroute.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_unroute_all
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_unroute_all = findDowncallHandle("uniffi_xcelerate_checksum_method_page_unroute_all", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_unroute_all() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_unroute_all.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_url
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_url = findDowncallHandle("uniffi_xcelerate_checksum_method_page_url", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_url() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_url.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_wait_for_event
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_wait_for_event = findDowncallHandle("uniffi_xcelerate_checksum_method_page_wait_for_event", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_wait_for_event() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_wait_for_event.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_wait_for_event_default
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_wait_for_event_default = findDowncallHandle("uniffi_xcelerate_checksum_method_page_wait_for_event_default", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_wait_for_event_default() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_wait_for_event_default.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_wait_for_function
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_wait_for_function = findDowncallHandle("uniffi_xcelerate_checksum_method_page_wait_for_function", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_wait_for_function() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_wait_for_function.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_wait_for_navigation
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_wait_for_navigation = findDowncallHandle("uniffi_xcelerate_checksum_method_page_wait_for_navigation", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_wait_for_navigation() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_wait_for_navigation.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_wait_for_selector
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_wait_for_selector = findDowncallHandle("uniffi_xcelerate_checksum_method_page_wait_for_selector", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_wait_for_selector() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_wait_for_selector.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_wait_for_xpath
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_wait_for_xpath = findDowncallHandle("uniffi_xcelerate_checksum_method_page_wait_for_xpath", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_wait_for_xpath() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_wait_for_xpath.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_window_id
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_window_id = findDowncallHandle("uniffi_xcelerate_checksum_method_page_window_id", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_window_id() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_window_id.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_window_position
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_window_position = findDowncallHandle("uniffi_xcelerate_checksum_method_page_window_position", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_window_position() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_window_position.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_window_rect
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_window_rect = findDowncallHandle("uniffi_xcelerate_checksum_method_page_window_rect", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_window_rect() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_window_rect.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_page_window_size
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_page_window_size = findDowncallHandle("uniffi_xcelerate_checksum_method_page_window_size", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_page_window_size() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_page_window_size.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_pluginhandle_invoke
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_pluginhandle_invoke = findDowncallHandle("uniffi_xcelerate_checksum_method_pluginhandle_invoke", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_pluginhandle_invoke() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_pluginhandle_invoke.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_pluginhandle_ops
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_pluginhandle_ops = findDowncallHandle("uniffi_xcelerate_checksum_method_pluginhandle_ops", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_pluginhandle_ops() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_pluginhandle_ops.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_method_pluginhandle_plugin_name
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_method_pluginhandle_plugin_name = findDowncallHandle("uniffi_xcelerate_checksum_method_pluginhandle_plugin_name", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_method_pluginhandle_plugin_name() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_method_pluginhandle_plugin_name.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // uniffi_xcelerate_checksum_constructor_browser_launch
    private static final java.lang.invoke.MethodHandle MH_uniffi_xcelerate_checksum_constructor_browser_launch = findDowncallHandle("uniffi_xcelerate_checksum_constructor_browser_launch", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_SHORT));

    static short uniffi_xcelerate_checksum_constructor_browser_launch() {
        try {
            return (short) MH_uniffi_xcelerate_checksum_constructor_browser_launch.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    // ffi_xcelerate_uniffi_contract_version
    private static final java.lang.invoke.MethodHandle MH_ffi_xcelerate_uniffi_contract_version = findDowncallHandle("ffi_xcelerate_uniffi_contract_version", java.lang.foreign.FunctionDescriptor.of(java.lang.foreign.ValueLayout.JAVA_INT));

    static int ffi_xcelerate_uniffi_contract_version() {
        try {
            return (int) MH_ffi_xcelerate_uniffi_contract_version.invokeExact();
        } catch (Throwable _ex) { throw new AssertionError("invokeExact failed", _ex); }
    }

    

    // Integrity checks and initialization must happen after all MethodHandle fields
    // are initialized (static fields are initialized in textual order in Java).
    static {
        NamespaceLibrary.uniffiCheckContractApiVersion();
        NamespaceLibrary.uniffiCheckApiChecksums();
        
        CLEANER = UniffiCleaner.create();
        }
}

