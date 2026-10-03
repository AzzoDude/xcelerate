package uniffi.xcelerate;

/**
 * A handle to an enabled plugin, exposed to every language.
 */
public interface PluginHandleInterface {
    
    /**
     * Invoke an op with a JSON-encoded argument object; returns JSON.
     */public java.util.concurrent.CompletableFuture<java.lang.String> invoke(java.lang.String op, java.lang.String argsJson) ;
    
    /**
     * The ops this plugin exposes.
     */public java.util.List<java.lang.String> ops();
    
    /**
     * The plugin's name.
     */public java.lang.String pluginName();
    
}

