mod activity_feed;
pub mod app;
mod clients_panel;
pub mod components;
pub mod events;
mod query_input;
mod quick_access;
mod servers_panel;
pub mod ui;

// Re-export key types for external use
pub use app::App;
pub use components::*;
pub use events::{Event, EventHandler};

use anyhow::Result;

/// Initialize the TUI application
pub async fn init() -> Result<App> {
    App::new().await
}
