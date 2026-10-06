//! Standalone MCP server binary. Equivalent to `xcelerate mcp`.

use mimalloc::MiMalloc;

/// The MCP server is allocation-heavy - every JSON-RPC frame is parsed and
/// re-encoded with serde_json - so it uses the same allocator as the CLI instead
/// of the system one.
///
/// This lives in the binary, not the library: the CLI also links `xcelerate_mcp`,
/// and two `#[global_allocator]` definitions in one artifact would not compile.
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    if let Err(error) = xcelerate_mcp::run_stdio().await {
        eprintln!("xcelerate-mcp: {error}");
        std::process::exit(1);
    }
}
