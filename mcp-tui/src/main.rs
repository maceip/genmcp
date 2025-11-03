use anyhow::Result;
use tracing::{error, info};
use tracing_subscriber;

use mcp_tui::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting MCP TUI - Next Generation Interface");

    // Initialize the application
    let mut app = App::new().await?;

    // Run the TUI
    if let Err(e) = app.run().await {
        error!("Application error: {}", e);
        return Err(e);
    }

    info!("MCP TUI shutdown complete");
    Ok(())
}
