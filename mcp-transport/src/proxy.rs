use anyhow::Result;
use mcp_common::{IpcMessage, ProxyId, ProxyInfo, ProxyStats, ProxyStatus};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

use crate::buffered_ipc_client::BufferedIpcClient;
use crate::stdio_handler::StdioHandler;
use crate::http_handler::HttpHandler;
use crate::transport_config::TransportConfig;

pub struct MCPProxy {
    id: ProxyId,
    name: String,
    transport_config: TransportConfig,
    stats: Arc<Mutex<ProxyStats>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl MCPProxy {
    pub async fn new(id: ProxyId, name: String, transport_config: TransportConfig) -> Result<Self> {
        let mut stats = ProxyStats::default();
        stats.proxy_id = id.clone();

        Ok(Self {
            id,
            name,
            transport_config,
            stats: Arc::new(Mutex::new(stats)),
            shutdown_tx: None,
        })
    }

    pub async fn start(&mut self, ipc_socket_path: Option<&str>) -> Result<()> {
        info!("Starting MCP proxy: {}", self.name);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Create buffered IPC client (unless monitor is explicitly disabled)
        let buffered_client = if let Some(socket_path) = ipc_socket_path {
            info!(
                "Creating buffered IPC client for monitor at {}",
                socket_path
            );
            Some(Arc::new(
                BufferedIpcClient::new(socket_path.to_string()).await,
            ))
        } else {
            info!("Running in standalone mode (monitor disabled)");
            None
        };

        // Send proxy started message
        if let Some(ref client) = buffered_client {
            let proxy_info = ProxyInfo {
                id: self.id.clone(),
                name: self.name.clone(),
                listen_address: "proxy".to_string(),
                target_command: vec![self.transport_config.display_target()],
                status: ProxyStatus::Starting,
                stats: self.stats.lock().await.clone(),
                transport_type: self.transport_config.transport_type(),
            };

            if let Err(e) = client.send(IpcMessage::ProxyStarted(proxy_info)).await {
                warn!("Failed to send proxy started message: {}", e);
            }
        }

        // Handle transport-specific logic
        match &self.transport_config {
            TransportConfig::Stdio { .. } => {
                // Start MCP server process
                let mut child = self.start_mcp_server().await?;

                // Create STDIO handler
                let mut handler =
                    StdioHandler::new(self.id.clone(), self.stats.clone(), buffered_client.clone()).await?;

                // Handle STDIO communication
                let result = handler.handle_communication(&mut child, shutdown_rx).await;

                // Clean up
                info!("Proxy {} shutting down", self.name);
                if let Err(e) = child.kill().await {
                    warn!("Failed to kill MCP server process: {}", e);
                }

                // Send proxy stopped message and shutdown buffered client
                if let Some(client) = buffered_client {
                    if let Err(e) = client.send(IpcMessage::ProxyStopped(self.id.clone())).await {
                        warn!("Failed to send proxy stopped message: {}", e);
                    }
                    // Take the client out of the Arc and shutdown
                    if let Ok(client) = Arc::try_unwrap(client) {
                        client.shutdown().await;
                    }
                }

                result
            }
            TransportConfig::HttpSse { .. } | TransportConfig::HttpStream { .. } => {
                // Create HTTP handler
                let mut handler =
                    HttpHandler::new(self.id.clone(), self.stats.clone(), buffered_client.clone()).await?;

                // Handle HTTP communication
                let result = handler.handle_communication(&self.transport_config, shutdown_rx).await;

                // Clean up
                info!("HTTP proxy {} shutting down", self.name);

                // Send proxy stopped message and shutdown buffered client
                if let Some(client) = buffered_client {
                    if let Err(e) = client.send(IpcMessage::ProxyStopped(self.id.clone())).await {
                        warn!("Failed to send proxy stopped message: {}", e);
                    }
                    // Take the client out of the Arc and shutdown
                    if let Ok(client) = Arc::try_unwrap(client) {
                        client.shutdown().await;
                    }
                }

                result
            }
        }
    }

    async fn start_mcp_server(&self) -> Result<Child> {
        let (command, use_shell) = match &self.transport_config {
            TransportConfig::Stdio { command, use_shell } => (command, use_shell),
            _ => return Err(anyhow::anyhow!("start_mcp_server only works for stdio transport")),
        };

        if command.is_empty() {
            return Err(anyhow::anyhow!("No command specified"));
        }

        let child = if *use_shell {
            // Use shell to execute the command
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        } else {
            // Parse command and arguments
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                return Err(anyhow::anyhow!("Empty command"));
            }

            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }

            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        info!("Started MCP server process: {}", command);
        Ok(child)
    }
}
