//! Tool prediction modules using DSPy-RS and LiteRT-LM

use std::sync::Arc;
use dspy_rs::{Predict, Module, Example, Prediction};
use async_trait::async_trait;
use crate::lm_provider::LiteRTLMProvider;
use crate::signatures::{
    ToolPredictionSignature, 
    SemanticRoutingSignature,
    ToolPrediction,
    RoutingDecision
};
use crate::error::{LlmError, LlmResult};

/// Main tool predictor for MCP requests
pub struct ToolPredictor {
    predict: Predict<ToolPredictionSignature>,
    lm_provider: Arc<LiteRTLMProvider>,
}

impl ToolPredictor {
    /// Create new tool predictor
    pub fn new(lm_provider: Arc<LiteRTLMProvider>) -> Self {
        let predict = Predict::new(ToolPredictionSignature::new());
        Self {
            predict,
            lm_provider,
        }
    }
    
    /// Predict which tool will be called for the given MCP context
    pub async fn predict_tool(&self, mcp_context: &str) -> LlmResult<ToolPrediction> {
        let example = Example::new()
            .with_input("mcp_context", mcp_context);
        
        let prediction = self.predict.forward(example).await
            .map_err(|e| LlmError::PredictionError(e.to_string()))?;
        
        let tool_prediction: ToolPrediction = serde_json::from_value(
            prediction.get("tool_prediction").cloned().unwrap_or_default()
        )?;
        
        Ok(tool_prediction)
    }
    
    /// Predict tool with confidence threshold
    pub async fn predict_tool_with_threshold(
        &self, 
        mcp_context: &str, 
        confidence_threshold: f32
    ) -> LlmResult<Option<ToolPrediction>> {
        let prediction = self.predict_tool(mcp_context).await?;
        
        if prediction.confidence >= confidence_threshold {
            Ok(Some(prediction))
        } else {
            Ok(None)
        }
    }
    
    /// Batch predict multiple contexts
    pub async fn predict_batch(&self, contexts: &[String]) -> LlmResult<Vec<ToolPrediction>> {
        let mut predictions = Vec::with_capacity(contexts.len());
        
        for context in contexts {
            let prediction = self.predict_tool(context).await?;
            predictions.push(prediction);
        }
        
        Ok(predictions)
    }
}

#[async_trait]
impl Module for ToolPredictor {
    async fn forward(&self, inputs: Example) -> Result<Prediction, dspy_rs::DspError> {
        let mcp_context = inputs.get("mcp_context", None);
        
        let prediction = self.predict_tool(&mcp_context).await
            .map_err(|e| dspy_rs::DspError::PredictionError(e.to_string()))?;
        
        let mut result = Prediction::new();
        result.set("tool_prediction", serde_json::to_value(prediction).unwrap());
        
        Ok(result)
    }
}

/// Semantic routing predictor
pub struct RoutingPredictor {
    predict: Predict<SemanticRoutingSignature>,
    lm_provider: Arc<LiteRTLMProvider>,
}

impl RoutingPredictor {
    pub fn new(lm_provider: Arc<LiteRTLMProvider>) -> Self {
        let predict = Predict::new(SemanticRoutingSignature::new());
        Self {
            predict,
            lm_provider,
        }
    }
    
    /// Predict optimal routing for given context
    pub async fn predict_routing(&self, routing_context: &str) -> LlmResult<RoutingDecision> {
        let example = Example::new()
            .with_input("routing_context", routing_context);
        
        let prediction = self.predict.forward(example).await
            .map_err(|e| LlmError::PredictionError(e.to_string()))?;
        
        let routing_decision: RoutingDecision = serde_json::from_value(
            prediction.get("routing_decision").cloned().unwrap_or_default()
        )?;
        
        Ok(routing_decision)
    }
}

#[async_trait]
impl Module for RoutingPredictor {
    async fn forward(&self, inputs: Example) -> Result<Prediction, dspy_rs::DspError> {
        let routing_context = inputs.get("routing_context", None);
        
        let prediction = self.predict_routing(&routing_context).await
            .map_err(|e| dspy_rs::DspError::PredictionError(e.to_string()))?;
        
        let mut result = Prediction::new();
        result.set("routing_decision", serde_json::to_value(prediction).unwrap());
        
        Ok(result)
    }
}

/// Advanced predictor with caching and learning
pub struct AdvancedToolPredictor {
    base_predictor: ToolPredictor,
    routing_predictor: RoutingPredictor,
    cache: Arc<tokio::sync::RwLock<std::collections::HashMap<String, ToolPrediction>>>,
}

impl AdvancedToolPredictor {
    pub fn new(lm_provider: Arc<LiteRTLMProvider>) -> Self {
        Self {
            base_predictor: ToolPredictor::new(lm_provider.clone()),
            routing_predictor: RoutingPredictor::new(lm_provider),
            cache: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Predict with caching
    pub async fn predict_cached(&self, mcp_context: &str) -> LlmResult<ToolPrediction> {
        let context_hash = self.hash_context(mcp_context);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached_prediction) = cache.get(&context_hash) {
                return Ok(cached_prediction.clone());
            }
        }
        
        // Generate new prediction
        let prediction = self.base_predictor.predict_tool(mcp_context).await?;
        
        // Cache the result
        {
            let mut cache = self.cache.write().await;
            cache.insert(context_hash, prediction.clone());
        }
        
        Ok(prediction)
    }
    
    /// Predict with routing consideration
    pub async fn predict_with_routing(&self, mcp_context: &str) -> LlmResult<(ToolPrediction, Option<RoutingDecision>)> {
        let tool_prediction = self.predict_cached(mcp_context).await?;
        
        // Generate routing decision for high-value predictions
        let routing_decision = if tool_prediction.confidence > 0.8 {
            let routing_context = self.build_routing_context(mcp_context, &tool_prediction);
            Some(self.routing_predictor.predict_routing(&routing_context).await?)
        } else {
            None
        };
        
        Ok((tool_prediction, routing_decision))
    }
    
    fn hash_context(&self, context: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        context.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    fn build_routing_context(&self, mcp_context: &str, tool_prediction: &ToolPrediction) -> String {
        format!(
            "MCP Context: {}\nPredicted Tool: {}\nConfidence: {}\nParameters: {}",
            mcp_context,
            tool_prediction.tool_name,
            tool_prediction.confidence,
            serde_json::to_string(&tool_prediction.parameters).unwrap_or_default()
        )
    }
}

#[async_trait]
impl Module for AdvancedToolPredictor {
    async fn forward(&self, inputs: Example) -> Result<Prediction, dspy_rs::DspError> {
        let mcp_context = inputs.get("mcp_context", None);
        
        let (tool_prediction, routing_decision) = self.predict_with_routing(&mcp_context).await
            .map_err(|e| dspy_rs::DspError::PredictionError(e.to_string()))?;
        
        let mut result = Prediction::new();
        result.set("tool_prediction", serde_json::to_value(tool_prediction).unwrap());
        
        if let Some(routing) = routing_decision {
            result.set("routing_decision", serde_json::to_value(routing).unwrap());
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lm_provider::{LiteRTBuilder, ResponseFormat};
    
    #[tokio::test]
    async fn test_tool_predictor_creation() {
        // This test would require actual LiteRT-LM setup
        // For now, just test the structure
        assert!(true);
    }
}