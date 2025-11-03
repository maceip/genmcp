use anyhow::Result;
use mcp_common::{IpcMessage, InterceptorInfo, InterceptorManagerInfo, LogEntry, LogLevel, ProxyId, ProxyStats};
use mcp_core::interceptor::{InterceptorManager, MessageDirection};
use mcp_core::messages::JsonRpcMessage;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Child;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::buffered_ipc_client::BufferedIpcClient;

pub struct StdioHandler {
    proxy_id: ProxyId,
    stats: Arc<Mutex<ProxyStats>>,
    ipc_client: Option<Arc<BufferedIpcClient>>,
    stats_interval: tokio::time::Interval,
    interceptor_manager: Arc<InterceptorManager>,
}

impl StdioHandler {
    pub async fn new(
        proxy_id: ProxyId,
        stats: Arc<Mutex<ProxyStats>>,
        ipc_client: Option<Arc<BufferedIpcClient>>,
    ) -> Result<Self> {
        use crate::interceptors::{TransformInterceptor, TransformRule, TransformOperation};
        use serde_json::json;

        let manager = InterceptorManager::new();

        // Replace "santa" with current timestamp in tool calls
        let transformer = TransformInterceptor::new();
        transformer.add_rule(TransformRule {
            name: "replace-santa-with-timestamp".to_string(),
            method_pattern: "tools/call".to_string(),
            path: "arguments.message".to_string(),
            operation: TransformOperation::Set {
                value: json!(chrono::Utc::now().to_rfc3339()),
            },
        }).await;
        manager.add_interceptor(Arc::new(transformer)).await;

        Self::with_interceptors(
            proxy_id,
            stats,
            ipc_client,
            Arc::new(manager),
        )
        .await
    }

    pub async fn with_interceptors(
        proxy_id: ProxyId,
        stats: Arc<Mutex<ProxyStats>>,
        ipc_client: Option<Arc<BufferedIpcClient>>,
        interceptor_manager: Arc<InterceptorManager>,
    ) -> Result<Self> {
        let stats_interval = interval(Duration::from_secs(1));

        Ok(Self {
            proxy_id,
            stats,
            ipc_client,
            stats_interval,
            interceptor_manager,
        })
    }

    /// Get the interceptor manager for this handler
    pub fn interceptor_manager(&self) -> &Arc<InterceptorManager> {
        &self.interceptor_manager
    }

    pub async fn handle_communication(
        &mut self,
        child: &mut Child,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get child stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get child stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get child stderr"))?;

        let mut child_stdin = BufWriter::new(stdin);
        let mut child_stdout = BufReader::new(stdout);
        let mut child_stderr = BufReader::new(stderr);

        let mut user_stdin = BufReader::new(tokio::io::stdin());
        let mut user_stdout = tokio::io::stdout();

        // Channels removed - not needed for direct STDIO handling

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal");
                    break;
                }

                // Handle stats updates
                _ = self.stats_interval.tick() => {
                    if let Some(ref client) = self.ipc_client {
                        // Send proxy stats
                        let stats = self.stats.lock().await.clone();
                        if let Err(e) = client.send(IpcMessage::StatsUpdate(stats)).await {
                            warn!("Failed to send stats update: {}", e);
                        }

                        // Send interceptor stats
                        let interceptor_stats = self.get_interceptor_stats().await;
                        if let Err(e) = client.send(IpcMessage::InterceptorStats {
                            proxy_id: self.proxy_id.clone(),
                            stats: interceptor_stats,
                        }).await {
                            warn!("Failed to send interceptor stats: {}", e);
                        }
                    }
                }

                // Read from user stdin and forward to child
                result = async {
                    let mut input = String::new();
                    let bytes_read = user_stdin.read_line(&mut input).await?;
                    Ok::<(usize, String), std::io::Error>((bytes_read, input))
                } => {
                    match result {
                        Ok((0, _)) => break, // EOF
                        Ok((_, input)) => {
                            // Process through interceptors
                            let (processed_input, modified) = match self.process_outgoing(&input).await {
                                Ok(result) => result,
                                Err(e) => {
                                    warn!("Message blocked or failed processing: {}", e);
                                    // Log the blocked message
                                    self.log_request(&input, false).await;
                                    {
                                        let mut stats = self.stats.lock().await;
                                        stats.failed_requests += 1;
                                    }
                                    continue; // Skip sending to child
                                }
                            };

                            self.log_request(&processed_input, modified).await;

                            if let Err(e) = child_stdin.write_all(processed_input.as_bytes()).await {
                                error!("Failed to write to child stdin: {}", e);
                                break;
                            }
                            if let Err(e) = child_stdin.flush().await {
                                error!("Failed to flush child stdin: {}", e);
                                break;
                            }

                            // Update stats
                            {
                                let mut stats = self.stats.lock().await;
                                stats.total_requests += 1;
                                stats.bytes_transferred += processed_input.len() as u64;
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from user stdin: {}", e);
                            break;
                        }
                    }
                }

                // Read from child stdout and forward to user
                result = async {
                    let mut output = String::new();
                    let bytes_read = child_stdout.read_line(&mut output).await?;
                    Ok::<(usize, String), std::io::Error>((bytes_read, output))
                } => {
                    match result {
                        Ok((0, _)) => {
                            info!("Child stdout closed");
                            break;
                        }
                        Ok((_, output)) => {
                            // Process through interceptors
                            let (processed_output, modified) = match self.process_incoming(&output).await {
                                Ok(result) => result,
                                Err(e) => {
                                    warn!("Message blocked or failed processing: {}", e);
                                    // Log the blocked message
                                    self.log_response(&output, false).await;
                                    {
                                        let mut stats = self.stats.lock().await;
                                        stats.failed_requests += 1;
                                    }
                                    continue; // Skip sending to user
                                }
                            };

                            self.log_response(&processed_output, modified).await;

                            if let Err(e) = user_stdout.write_all(processed_output.as_bytes()).await {
                                error!("Failed to write to user stdout: {}", e);
                                break;
                            }
                            if let Err(e) = user_stdout.flush().await {
                                error!("Failed to flush user stdout: {}", e);
                                break;
                            }

                            // Update stats
                            {
                                let mut stats = self.stats.lock().await;
                                stats.successful_requests += 1;
                                stats.bytes_transferred += processed_output.len() as u64;
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from child stdout: {}", e);
                            {
                                let mut stats = self.stats.lock().await;
                                stats.failed_requests += 1;
                            }
                            break;
                        }
                    }
                }

                // Read from child stderr and log as errors
                result = async {
                    let mut error_msg = String::new();
                    let bytes_read = child_stderr.read_line(&mut error_msg).await?;
                    Ok::<(usize, String), std::io::Error>((bytes_read, error_msg))
                } => {
                    match result {
                        Ok((0, _)) => {
                            debug!("Child stderr closed");
                        }
                        Ok((_, error_msg)) => {
                            self.log_error(&error_msg).await;

                            // Also forward stderr to user stderr
                            if let Err(e) = tokio::io::stderr().write_all(error_msg.as_bytes()).await {
                                warn!("Failed to write child stderr to user stderr: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from child stderr: {}", e);
                        }
                    }
                }

                // Check if child process has exited
                status = child.wait() => {
                    match status {
                        Ok(exit_status) => {
                            info!("Child process exited with status: {}", exit_status);
                            if !exit_status.success() {
                                let mut stats = self.stats.lock().await;
                                stats.failed_requests += 1;
                            }
                        }
                        Err(e) => {
                            error!("Failed to wait for child process: {}", e);
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Process an outgoing message (client -> server) through interceptors
    async fn process_outgoing(&self, content: &str) -> Result<(String, bool)> {
        // Try to parse as JSON-RPC message
        match serde_json::from_str::<JsonRpcMessage>(content.trim()) {
            Ok(message) => {
                // Process through interceptors
                match self
                    .interceptor_manager
                    .process_message(message, MessageDirection::Outgoing)
                    .await
                {
                    Ok(result) => {
                        if result.block {
                            warn!("Message blocked by interceptor: {:?}", result.reasoning);
                            return Err(anyhow::anyhow!(
                                "Message blocked: {}",
                                result.reasoning.unwrap_or_default()
                            ));
                        }

                        let modified_content = serde_json::to_string(&result.message)?;
                        Ok((modified_content + "\n", result.modified))
                    }
                    Err(e) => {
                        warn!("Interceptor processing failed: {}", e);
                        // Fall back to original message
                        Ok((content.to_string(), false))
                    }
                }
            }
            Err(_) => {
                // Not valid JSON-RPC, pass through unchanged
                Ok((content.to_string(), false))
            }
        }
    }

    /// Process an incoming message (server -> client) through interceptors
    async fn process_incoming(&self, content: &str) -> Result<(String, bool)> {
        // Try to parse as JSON-RPC message
        match serde_json::from_str::<JsonRpcMessage>(content.trim()) {
            Ok(message) => {
                // Process through interceptors
                match self
                    .interceptor_manager
                    .process_message(message, MessageDirection::Incoming)
                    .await
                {
                    Ok(result) => {
                        if result.block {
                            warn!("Message blocked by interceptor: {:?}", result.reasoning);
                            return Err(anyhow::anyhow!(
                                "Message blocked: {}",
                                result.reasoning.unwrap_or_default()
                            ));
                        }

                        let modified_content = serde_json::to_string(&result.message)?;
                        Ok((modified_content + "\n", result.modified))
                    }
                    Err(e) => {
                        warn!("Interceptor processing failed: {}", e);
                        // Fall back to original message
                        Ok((content.to_string(), false))
                    }
                }
            }
            Err(_) => {
                // Not valid JSON-RPC, pass through unchanged
                Ok((content.to_string(), false))
            }
        }
    }

    async fn log_request(&mut self, content: &str, modified: bool) {
        let prefix = if modified { "→ [MODIFIED]" } else { "→" };
        let log_entry = LogEntry::new(
            LogLevel::Request,
            format!("{} {}", prefix, content.trim()),
            self.proxy_id.clone(),
        );

        if let Some(ref client) = self.ipc_client {
            if let Err(e) = client.send(IpcMessage::LogEntry(log_entry)).await {
                warn!("Failed to send log entry: {}", e);
            }
        }

        debug!("Request{}: {}", if modified { " (modified)" } else { "" }, content.trim());
    }

    async fn log_response(&mut self, content: &str, modified: bool) {
        let prefix = if modified { "← [MODIFIED]" } else { "←" };
        let log_entry = LogEntry::new(
            LogLevel::Response,
            format!("{} {}", prefix, content.trim()),
            self.proxy_id.clone(),
        );

        if let Some(ref client) = self.ipc_client {
            if let Err(e) = client.send(IpcMessage::LogEntry(log_entry)).await {
                warn!("Failed to send log entry: {}", e);
            }
        }

        debug!("Response{}: {}", if modified { " (modified)" } else { "" }, content.trim());
    }

    async fn log_error(&mut self, content: &str) {
        let log_entry = LogEntry::new(
            LogLevel::Error,
            format!("stderr: {}", content.trim()),
            self.proxy_id.clone(),
        );

        if let Some(ref client) = self.ipc_client {
            if let Err(e) = client.send(IpcMessage::LogEntry(log_entry)).await {
                warn!("Failed to send log entry: {}", e);
            }
        }

        error!("Child stderr: {}", content.trim());
    }

    /// Get interceptor statistics from the manager
    async fn get_interceptor_stats(&self) -> InterceptorManagerInfo {
        let manager_stats = self.interceptor_manager.get_stats().await;
        let interceptor_names = self.interceptor_manager.list_interceptors().await;

        let mut interceptors = Vec::new();
        for name in interceptor_names {
            // Note: We don't currently track enabled/disabled state per interceptor
            // This would require adding that capability to InterceptorManager
            interceptors.push(InterceptorInfo {
                name: name.clone(),
                priority: 0, // Would need to query this from the actual interceptor
                enabled: true, // Assume enabled for now
                total_intercepted: 0, // Would need per-interceptor tracking
                total_modified: 0,
                total_blocked: 0,
                avg_processing_time_ms: 0.0,
            });
        }

        InterceptorManagerInfo {
            total_messages_processed: manager_stats.total_messages_processed,
            total_modifications_made: manager_stats.total_modifications_made,
            total_messages_blocked: manager_stats.total_messages_blocked,
            avg_processing_time_ms: manager_stats.avg_processing_time_ms,
            messages_by_method: manager_stats.messages_by_method,
            interceptors,
        }
    }
}
