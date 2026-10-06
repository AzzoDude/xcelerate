package uniffi.xcelerate;

/**
 * Configuration for the Browser instance.
 */
public class BrowserConfig {
    /**
     * Whether to run the browser in headless mode.
     */
    private boolean headless;
    /**
     * Whether to run the browser as a detached process.
     */
    private boolean detached;
    /**
     * Optional path to the browser executable.
     */
    private java.lang.String executablePath;
    /**
     * External plugins to load at launch. Each entry is a path to a plugin
     * directory or a `plugin.json`.
     *
     * Xcelerate ships **no** plugins built into the core. Default-deny: no
     * plugin does anything unless it is listed here (or installed afterwards
     * with `Browser::use_plugin` / `Browser::load_plugin`).
     */
    private java.util.List<java.lang.String> plugins;

    public BrowserConfig(
        boolean headless, 
        boolean detached, 
        java.lang.String executablePath, 
        java.util.List<java.lang.String> plugins
    ) {
        
        this.headless = headless;
        
        this.detached = detached;
        
        this.executablePath = executablePath;
        
        this.plugins = plugins;
    }
    
    public boolean headless() {
        return this.headless;
    }
    
    public boolean detached() {
        return this.detached;
    }
    
    public java.lang.String executablePath() {
        return this.executablePath;
    }
    
    public java.util.List<java.lang.String> plugins() {
        return this.plugins;
    }
    public void setHeadless(boolean headless) {
        this.headless = headless;
    }
    public void setDetached(boolean detached) {
        this.detached = detached;
    }
    public void setExecutablePath(java.lang.String executablePath) {
        this.executablePath = executablePath;
    }
    public void setPlugins(java.util.List<java.lang.String> plugins) {
        this.plugins = plugins;
    }

    

    
    @Override
    public boolean equals(java.lang.Object other) {
        if (other instanceof BrowserConfig) {
            BrowserConfig t = (BrowserConfig) other;
            return (
              headless == t.headless && 
              
              detached == t.detached && 
              
              java.util.Objects.equals(executablePath, t.executablePath) && 
              
              java.util.Objects.equals(plugins, t.plugins)
              
            );
        };
        return false;
    }
    @Override
    public int hashCode() {
        int result = 17;
        result = 31 * result + java.lang.Boolean.hashCode(headless);
        result = 31 * result + java.lang.Boolean.hashCode(detached);
        result = 31 * result + java.util.Objects.hashCode(executablePath);
        result = 31 * result + java.util.Objects.hashCode(plugins);
        return result;
    }

    
    
    

}


