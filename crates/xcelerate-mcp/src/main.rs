//! Standalone MCP server binary. Equivalent to `xcelerate mcp`.

#[tokio::main]
async fn main() {
    if let Err(error) = xcelerate_mcp::run_stdio().await {
        eprintln!("xcelerate-mcp: {error}");
        std::process::exit(1);
    }
}
