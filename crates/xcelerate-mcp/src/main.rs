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

fn main() {
    // The tool dispatcher is a single, large async fn; its future - plus the
    // browser launch it drives - needs more stack than the platform's default
    // main thread reserves, and overflowing it aborts the process.
    let worker = std::thread::Builder::new()
        .name("xcelerate-mcp".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            else {
                eprintln!("xcelerate-mcp: could not start the async runtime");
                std::process::exit(1);
            };
            if let Err(error) = runtime.block_on(xcelerate_mcp::run_stdio()) {
                eprintln!("xcelerate-mcp: {error}");
                std::process::exit(1);
            }
        })
        .expect("spawn the xcelerate-mcp worker thread");
    let _ = worker.join();
}
