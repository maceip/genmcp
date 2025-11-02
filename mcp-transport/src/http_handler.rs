use anyhow::Result;
use mcp_common::{IpcMessage, LogEntry, LogLevel, ProxyId, ProxyStats};
use mcp_core::{McpClient, TransportConfig as McpTransportConfig};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info, warn};

use crate::buffered_ipc_client::BufferedIpcClient;
use crate::transport_config::TransportConfig;

pub struct HttpHandler {
    proxy_id: ProxyId,
    stats: Arc<Mutex<ProxyStats>>,
    ipc_client: Option<Arc<BufferedIpcClient>>,
}

impl HttpHandler {
    pub async fn new(
        proxy_id: ProxyId,
        stats: Arc<Mutex<ProxyStats>>,
        ipc_client: Option<Arc<BufferedIpcClient>>,
    ) -> Result<Self> {
        Ok(Self {
            proxy_id,
            stats,
            ipc_client,
        })
    }

    pub async fn handle_communication(
        &mut self,
        transport_config: &TransportConfig,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        info!("Starting HTTP handler");

        // Convert our TransportConfig to mcp-core's TransportConfig
        let mcp_config = match transport_config {
            TransportConfig::HttpSse { url, .. } => {
                info!("Connecting to HTTP+SSE server at {}", url);
                McpTransportConfig::http_sse(&url)?
            }
            TransportConfig::HttpStream { .. } => {
                return Err(anyhow::anyhow!(
                    "HTTP Stream transport not yet implemented. Use http-sse for now."
                ));
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "HttpHandler only supports HTTP transports"
                ))
            }
        };

        // Create MCP client
        let mut _client = McpClient::with_defaults(mcp_config).await?;

        // Log connection success
        self.log(LogLevel::Info, "Connected to HTTP server".to_string())
            .await;

        // For now, just wait for shutdown (full bidirectional communication coming in Stage 2)
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        info!("HTTP handler shutting down");
        Ok(())
    }

    async fn log(&self, level: LogLevel, message: String) {
        if let Some(ref client) = self.ipc_client {
            let log_entry = LogEntry::new(level, message, self.proxy_id.clone());
            if let Err(e) = client.send(IpcMessage::LogEntry(log_entry)).await {
                warn!("Failed to send log entry: {}", e);
            }
        }
    }
}
