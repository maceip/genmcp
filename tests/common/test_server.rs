//! Common test MCP server implementation

use std::sync::Arc;
use tokio::sync::RwLock;
use mcp_common::types::{ProxySession, SessionId, LogEntry};
use mcp_core::{McpClient, ServerInfo};

/// A test MCP server for integration testing
pub struct TestMcpServer {
    pub sessions: Arc<RwLock<Vec<ProxySession>>>,
    pub port: u16,
}

impl TestMcpServer {
    pub fn new(port: u16) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(Vec::new())),
            port,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual test server startup
        println!("Test MCP server started on port {}", self.port);
        Ok(())
    }

    pub async fn stop(&self) {
        // TODO: Implement graceful shutdown
        println!("Test MCP server stopped");
    }

    pub async fn add_test_session(&self, session_id: SessionId) {
        let session = ProxySession {
            id: session_id,
            name: "test-session".to_string(),
            transport_type: "stdio".to_string(),
            status: mcp_common::types::SessionStatus::Active,
            request_count: 0,
            last_activity: chrono::Utc::now(),
        };
        
        self.sessions.write().await.push(session);
    }

    pub async fn get_sessions(&self) -> Vec<ProxySession> {
        self.sessions.read().await.clone()
    }
}
