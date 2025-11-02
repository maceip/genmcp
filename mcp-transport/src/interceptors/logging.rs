//! Logging interceptor that captures all MCP traffic for debugging

use async_trait::async_trait;
use mcp_core::interceptor::{
    InterceptionResult, InterceptorStats, MessageContext, MessageInterceptor,
};
use mcp_core::McpResult;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Interceptor that logs all MCP messages for debugging and monitoring
pub struct LoggingInterceptor {
    name: String,
    stats: Arc<RwLock<InterceptorStats>>,
    /// Whether to log message content (can be verbose)
    log_content: bool,
}

impl LoggingInterceptor {
    /// Create a new logging interceptor
    pub fn new(log_content: bool) -> Self {
        Self {
            name: "LoggingInterceptor".to_string(),
            stats: Arc::new(RwLock::new(InterceptorStats::default())),
            log_content,
        }
    }
}

#[async_trait]
impl MessageInterceptor for LoggingInterceptor {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> u32 {
        // Run first (low priority) to log everything before modifications
        10
    }

    async fn should_intercept(&self, _context: &MessageContext) -> bool {
        // Log all messages
        true
    }

    async fn intercept(&self, context: MessageContext) -> McpResult<InterceptionResult> {
        let start = std::time::Instant::now();

        // Log based on message type
        let method = context.method().unwrap_or("unknown");
        let direction = format!("{:?}", context.direction);

        if self.log_content {
            info!(
                "[{}] {} - {} - {}",
                self.name,
                direction,
                method,
                serde_json::to_string(&context.message)?
            );
        } else {
            debug!(
                "[{}] {} - {} (id: {:?})",
                self.name,
                direction,
                method,
                context.id()
            );
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_intercepted += 1;
        stats.last_processed = Some(chrono::Utc::now());

        let elapsed = start.elapsed().as_millis() as f64;
        stats.avg_processing_time_ms =
            (stats.avg_processing_time_ms * (stats.total_intercepted - 1) as f64 + elapsed)
            / stats.total_intercepted as f64;

        // Pass through without modification
        Ok(InterceptionResult::pass_through(context.message))
    }

    async fn get_stats(&self) -> InterceptorStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_core::interceptor::MessageDirection;
    use mcp_core::messages::{JsonRpcMessage, JsonRpcRequest, RequestId};
    use serde_json::json;

    #[tokio::test]
    async fn test_logging_interceptor_passes_through() {
        let interceptor = LoggingInterceptor::new(false);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::from(1i64),
            method: "tools/list".to_string(),
            params: Some(json!({})),
        };

        let context = MessageContext::new(
            JsonRpcMessage::Request(request.clone()),
            MessageDirection::Outgoing,
        );

        let result = interceptor.intercept(context).await.unwrap();

        assert!(!result.modified);
        assert!(!result.block);

        // Verify stats updated
        let stats = interceptor.get_stats().await;
        assert_eq!(stats.total_intercepted, 1);
        assert_eq!(stats.total_modified, 0);
    }

    #[tokio::test]
    async fn test_logging_interceptor_stats() {
        let interceptor = LoggingInterceptor::new(true);

        // Process multiple messages
        for i in 0..5 {
            let request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: RequestId::from(i),
                method: "test/method".to_string(),
                params: None,
            };

            let context = MessageContext::new(
                JsonRpcMessage::Request(request),
                MessageDirection::Outgoing,
            );

            interceptor.intercept(context).await.unwrap();
        }

        let stats = interceptor.get_stats().await;
        assert_eq!(stats.total_intercepted, 5);
        assert!(stats.last_processed.is_some());
        assert!(stats.avg_processing_time_ms >= 0.0);
    }
}
