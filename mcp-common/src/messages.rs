use crate::{
    AppliedTransformation, ClientId, ClientInfo, GatewayMetrics, GatewayState, HealthMetrics,
    LogEntry, MessageFlow, ProxyId, ProxyInfo, ProxySession, ProxyStats, RoutingDecision,
    RoutingRule, ServerId, ServerInfo, SessionId, TransformationRule,
};
use crate::{JsonRpcRequest, JsonRpcResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for an interceptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptorInfo {
    pub name: String,
    pub priority: u32,
    pub enabled: bool,
    pub total_intercepted: u64,
    pub total_modified: u64,
    pub total_blocked: u64,
    pub avg_processing_time_ms: f64,
}

/// Manager-level interceptor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptorManagerInfo {
    pub total_messages_processed: u64,
    pub total_modifications_made: u64,
    pub total_messages_blocked: u64,
    pub avg_processing_time_ms: f64,
    pub messages_by_method: HashMap<String, u64>,
    pub interceptors: Vec<InterceptorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    // Proxy -> Monitor messages
    ProxyStarted(ProxyInfo),
    ProxyStopped(ProxyId),
    LogEntry(LogEntry),
    StatsUpdate(ProxyStats),
    InterceptorStats {
        proxy_id: ProxyId,
        stats: InterceptorManagerInfo,
    },
    ClientConnected(ClientInfo),
    ClientDisconnected(ClientId),
    ClientUpdated(ClientInfo),
    ClientRequest {
        client_id: ClientId,
        request: JsonRpcRequest,
        session_id: Option<SessionId>,
    },
    ServerConnected(ServerInfo),
    ServerDisconnected(ServerId),
    ServerUpdated(ServerInfo),
    ServerResponse {
        server_id: ServerId,
        response: JsonRpcResponse,
        session_id: Option<SessionId>,
    },
    ServerHealthUpdate {
        server_id: ServerId,
        metrics: HealthMetrics,
    },
    SessionStarted(ProxySession),
    SessionUpdated(ProxySession),
    SessionEnded(SessionId),
    TransformationRules(Vec<TransformationRule>),
    TransformationApplied {
        session_id: SessionId,
        transformation: AppliedTransformation,
    },
    RoutingRules(Vec<RoutingRule>),
    RoutingDecision(RoutingDecision),
    GatewayStateUpdated(GatewayState),
    GatewayMetrics(GatewayMetrics),
    MessageFlowUpdate(MessageFlow),

    // Monitor -> Proxy messages
    GetStatus(ProxyId),
    GetLogs {
        proxy_id: ProxyId,
        limit: Option<usize>,
    },
    Shutdown(ProxyId),
    ToggleInterceptor {
        proxy_id: ProxyId,
        interceptor_name: String,
    },

    // Bidirectional messages
    Ping,
    Pong,

    // Error handling
    Error {
        message: String,
        proxy_id: Option<ProxyId>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEnvelope {
    pub message: IpcMessage,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<uuid::Uuid>,
}
