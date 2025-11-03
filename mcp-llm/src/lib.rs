//! # MCP LLM Integration
//!
//! [mcp-llm](cci:7://file:///Users/rpm/assist-mcp/mcp-llm:0:0-0:0) provides intelligent LLM integration for assist-mcp, featuring:
//! - LiteRT-LM C++ bindings via bindgen
//! - DSPy-RS integration for structured predictions
//! - SQLite-backed routing and optimization
//! - GEPA prompt optimization
//! - Real-time tool prediction and routing

// Include generated bindings
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;

pub mod error;
pub mod litert_wrapper;
pub mod session_management;
pub mod conversation_context;
pub mod dspy_signatures;
pub mod gepa_optimizer;
pub mod predictors;
pub mod lm_provider;
pub mod routing_modes;
pub mod metrics;
pub mod interceptor;

pub mod routing_modes;
pub mod metrics;
pub mod interceptor;

// Re-export main types
pub use error::{LlmError, LlmResult};
pub use litert_wrapper::{LiteRTEngine, LiteRTSession, LiteRTBackend};
pub use session_management::{SessionManager, SessionPredictionContext, SessionPrediction};
pub use conversation_context::{ConversationContextBuilder, ConversationAnalyzer};
pub use dspy_signatures::{ToolPrediction, ToolPredictionSignature};
pub use predictors::{ToolPredictor, AdvancedToolPredictor};
pub use gepa_optimizer::GEPAOptimizer;

/// High-level LLM Manager for easy use
pub struct LlmManager {
    engine: LiteRTEngine,
    session_manager: SessionManager,
}

impl LlmManager {
    pub async fn new(model_path: &str) -> LlmResult<Self> {
        let engine = LiteRTEngine::new(model_path, LiteRTBackend::Cpu)?;
        let predictor = Arc::new(AdvancedToolPredictor::new()?);
        let gepa_optimizer = Arc::new(GEPAOptimizer::new()?);
        let session_manager = SessionManager::new(predictor, gepa_optimizer);
        
        Ok(Self {
            engine,
            session_manager,
        })
    }
}

/// Simple config for LlmManager
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub model_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bindings_available() {
        // Test that bindings are generated and available
    }
}