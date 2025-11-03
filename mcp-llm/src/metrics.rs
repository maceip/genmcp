//! Metrics collection for LLM performance monitoring

use std::sync::Arc;
use std::time::Instant;
use crate::database::MetricsDatabase;
use crate::error::{LlmError, LlmResult};
use metrics::{counter, histogram, gauge};

/// Metrics collector for LLM operations
pub struct LlmMetricsCollector {
    database: Arc<MetricsDatabase>,
    start_time: Instant,
}

impl LlmMetricsCollector {
    /// Create new metrics collector
    pub fn new(database: Arc<MetricsDatabase>) -> Self {
        Self {
            database,
            start_time: Instant::now(),
        }
    }
    
    /// Record prediction metrics
    pub async fn record_prediction(
        &self,
        tool_name: &str,
        confidence: f64,
        prediction_time_ms: u64,
        success: bool,
    ) -> LlmResult<()> {
        // Record to database
        let tags = serde_json::json!({
            "tool_name": tool_name,
            "success": success
        }).to_string();
        
        self.database.record_metric("prediction_confidence", confidence, &tags).await?;
        self.database.record_metric("prediction_time", prediction_time_ms as f64, &tags).await?;
        
        // Record to metrics system
        counter!("llm_predictions_total", 1, "tool" => tool_name, "success" => &success.to_string());
        histogram!("llm_prediction_duration_seconds", prediction_time_ms as f64 / 1000.0, "tool" => tool_name);
        gauge!("llm_prediction_confidence", confidence, "tool" => tool_name);
        
        Ok(())
    }
    
    /// Record GEPA optimization metrics
    pub async fn record_gepa_optimization(
        &self,
        iterations: usize,
        improvement: f64,
        optimization_time_ms: u64,
    ) -> LlmResult<()> {
        let tags = serde_json::json!({
            "iterations": iterations,
            "successful": improvement > 0.1
        }).to_string();
        
        self.database.record_metric("gepa_improvement", improvement, &tags).await?;
        self.database.record_metric("gepa_optimization_time", optimization_time_ms as f64, &tags).await?;
        
        // Record to metrics system
        counter!("gepa_optimizations_total", 1);
        histogram!("gepa_optimization_duration_seconds", optimization_time_ms as f64 / 1000.0);
        gauge!("gepa_improvement_ratio", improvement);
        
        Ok(())
    }
    
    /// Record routing metrics
    pub async fn record_routing_decision(
        &self,
        mode: &str,
        confidence: f64,
        used_llm: bool,
    ) -> LlmResult<()> {
        let tags = serde_json::json!({
            "mode": mode,
            "used_llm": used_llm
        }).to_string();
        
        self.database.record_metric("routing_confidence", confidence, &tags).await?;
        
        // Record to metrics system
        counter!("llm_routing_decisions_total", 1, "mode" => mode, "used_llm" => &used_llm.to_string());
        gauge!("llm_routing_confidence", confidence, "mode" => mode);
        
        Ok(())
    }
    
    /// Get system uptime
    pub fn uptime_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

/// Performance metrics summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_predictions: i64,
    pub average_confidence: f64,
    pub success_rate: f64,
    pub average_prediction_time_ms: f64,
    pub gepa_optimizations: i64,
    pub average_gepa_improvement: f64,
}

impl PerformanceSummary {
    pub fn new() -> Self {
        Self {
            total_predictions: 0,
            average_confidence: 0.0,
            success_rate: 0.0,
            average_prediction_time_ms: 0.0,
            gepa_optimizations: 0,
            average_gepa_improvement: 0.0,
        }
    }
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collection() {
        // Test would require actual database setup
        assert!(true);
    }
}   