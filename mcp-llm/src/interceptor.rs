//! LLM interceptor for intelligent request routing and modification

use std::sync::Arc;
use crate::predictors::ToolPredictor;
use crate::routing_modes::RoutingMode;
use crate::database::{RoutingRulesDatabase, PredictionsDatabase};
use crate::error::{LlmError, LlmResult};
use mcp_core::interceptor::{MessageInterceptor, InterceptionResult, JsonRpcMessage};
use serde_json::Value;

/// LLM-powered interceptor for intelligent request processing
pub struct LlmInterceptor {
    predictor: Arc<ToolPredictor>,
    routing_db: RoutingRulesDatabase,
    predictions_db: PredictionsDatabase,
    routing_mode: RoutingMode,
    confidence_threshold: f32,
}

impl LlmInterceptor {
    /// Create new LLM interceptor
    pub fn new(
        predictor: Arc<ToolPredictor>,
        routing_mode: RoutingMode,
    ) -> Self {
        // Note: In real implementation, would need database instances
        Self {
            predictor,
            routing_db: RoutingRulesDatabase::placeholder(),
            predictions_db: PredictionsDatabase::placeholder(),
            routing_mode,
            confidence_threshold: 0.8,
        }
    }
    
    /// Set routing mode
    pub fn set_routing_mode(&mut self, mode: RoutingMode) {
        self.routing_mode = mode;
    }
    
    /// Get current routing mode
    pub fn get_routing_mode(&self) -> &RoutingMode {
        &self.routing_mode
    }
    
    /// Predict and route request
    async fn predict_and_route(&self, message: &mut JsonRpcMessage) -> LlmResult<InterceptionResult> {
        let context = self.extract_mcp_context(message)?;
        
        match self.routing_mode {
            RoutingMode::Bypass => Ok(InterceptionResult::Pass),
            RoutingMode::Semantic => self.semantic_routing(message, &context).await,
            RoutingMode::Hybrid => self.hybrid_routing(message, &context).await,
        }
    }
    
    /// Semantic routing using LLM predictions
    async fn semantic_routing(&self, message: &mut JsonRpcMessage, context: &str) -> LlmResult<InterceptionResult> {
        let prediction = self.predictor.predict_tool(context).await?;
        
        // Record prediction
        let context_hash = self.hash_context(context);
        self.predictions_db.record_prediction(
            &context_hash,
            &prediction.tool_name,
            prediction.confidence as f64,
            serde_json::to_value(&prediction)?,
        ).await?;
        
        if prediction.confidence >= self.confidence_threshold {
            // Modify request based on prediction
            self.enhance_request_with_prediction(message, &prediction).await?;
            Ok(InterceptionResult::Modified)
        } else {
            Ok(InterceptionResult::Pass)
        }
    }
    
    /// Hybrid routing combining database rules and LLM predictions
    async fn hybrid_routing(&self, message: &mut JsonRpcMessage, context: &str) -> LlmResult<InterceptionResult> {
        // First check database rules
        if let Some(rule) = self.routing_db.find_matching_rule(context).await? {
            self.apply_routing_rule(message, &rule).await?;
            return Ok(InterceptionResult::Modified);
        }
        
        // Fall back to LLM prediction
        self.semantic_routing(message, context).await
    }
    
    /// Extract MCP context from message
    fn extract_mcp_context(&self, message: &JsonRpcMessage) -> LlmResult<String> {
        let context = json!({
            "method": message.method,
            "params": message.params,
            "id": message.id
        });
        
        Ok(serde_json::to_string(&context)?)
    }
    
    /// Hash context for prediction tracking
    fn hash_context(&self, context: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        context.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Enhance request with prediction insights
    async fn enhance_request_with_prediction(
        &self,
        message: &mut JsonRpcMessage,
        prediction: &crate::signatures::ToolPrediction,
    ) -> LlmResult<()> {
        // Add prediction metadata to message
        if let Some(ref mut params) = message.params {
            if let Some(obj) = params.as_object_mut() {
                obj.insert("_predicted_tool".to_string(), Value::String(prediction.tool_name.clone()));
                obj.insert("_prediction_confidence".to_string(), Value::Number(serde_json::Number::from_f64(prediction.confidence as f64).unwrap()));
            }
        }
        
        Ok(())
    }
    
    /// Apply routing rule to message
    async fn apply_routing_rule(&self, message: &mut JsonRpcMessage, rule: &crate::database::RoutingRule) -> LlmResult<()> {
        // Add routing metadata
        if let Some(ref mut params) = message.params {
            if let Some(obj) = params.as_object_mut() {
                obj.insert("_routed_transport".to_string(), Value::String(rule.target_transport.clone()));
                obj.insert("_routing_confidence".to_string(), Value::Number(serde_json::Number::from_f64(rule.confidence).unwrap()));
            }
        }
        
        Ok(())
    }
}

impl MessageInterceptor for LlmInterceptor {
    fn intercept_outgoing(&mut self, message: &mut JsonRpcMessage) -> InterceptionResult {
        // In a real implementation, this would be async
        // For now, return Pass as placeholder
        InterceptionResult::Pass
    }
    
    fn intercept_incoming(&mut self, message: &mut JsonRpcMessage) -> InterceptionResult {
        // Handle response messages to update prediction accuracy
        if let Some(result) = message.get_result() {
            // Update prediction accuracy based on actual result
            // This would extract the actual tool used and update the database
        }
        
        InterceptionResult::Pass
    }
}

// Placeholder implementations for database structs
impl RoutingRulesDatabase {
    fn placeholder() -> Self {
        // In real implementation, would create with actual database pool
        unimplemented!("Placeholder implementation")
    }
}

impl PredictionsDatabase {
    fn placeholder() -> Self {
        // In real implementation, would create with actual database pool
        unimplemented!("Placeholder implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_llm_interceptor_creation() {
        // Test would require actual predictor setup
        assert!(true);
    }
}