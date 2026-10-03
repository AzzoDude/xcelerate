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
     * Whether to apply stealth patches to the binary.
     */
    private boolean stealth;
    /**
     * Whether to run the browser as a detached process.
     */
    private boolean detached;
    /**
     * Optional path to the browser executable.
     */
    private java.lang.String executablePath;

    public BrowserConfig(
        boolean headless, 
        boolean stealth, 
        boolean detached, 
        java.lang.String executablePath
    ) {
        
        this.headless = headless;
        
        this.stealth = stealth;
        
        this.detached = detached;
        
        this.executablePath = executablePath;
    }
    
    public boolean headless() {
        return this.headless;
    }
    
    public boolean stealth() {
        return this.stealth;
    }
    
    public boolean detached() {
        return this.detached;
    }
    
    public java.lang.String executablePath() {
        return this.executablePath;
    }
    public void setHeadless(boolean headless) {
        this.headless = headless;
    }
    public void setStealth(boolean stealth) {
        this.stealth = stealth;
    }
    public void setDetached(boolean detached) {
        this.detached = detached;
    }
    public void setExecutablePath(java.lang.String executablePath) {
        this.executablePath = executablePath;
    }

    

    
    @Override
    public boolean equals(java.lang.Object other) {
        if (other instanceof BrowserConfig) {
            BrowserConfig t = (BrowserConfig) other;
            return (
              headless == t.headless && 
              
              stealth == t.stealth && 
              
              detached == t.detached && 
              
              java.util.Objects.equals(executablePath, t.executablePath)
              
            );
        };
        return false;
    }
    @Override
    public int hashCode() {
        int result = 17;
        result = 31 * result + java.lang.Boolean.hashCode(headless);
        result = 31 * result + java.lang.Boolean.hashCode(stealth);
        result = 31 * result + java.lang.Boolean.hashCode(detached);
        result = 31 * result + java.util.Objects.hashCode(executablePath);
        return result;
    }

    
    
    

}


