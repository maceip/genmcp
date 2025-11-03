//! Routing modes for LLM interceptor

use serde::{Deserialize, Serialize};

/// Routing mode for LLM interceptor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoutingMode {
    /// Pass through all requests without modification
    Bypass,
    /// Use LLM predictions for routing decisions
    Semantic,
    /// Combine database rules with LLM predictions
    Hybrid,
}

impl RoutingMode {
    /// Get display name for routing mode
    pub fn display_name(&self) -> &'static str {
        match self {
            RoutingMode::Bypass => "Bypass",
            RoutingMode::Semantic => "Semantic",
            RoutingMode::Hybrid => "Hybrid",
        }
    }
    
    /// Get icon for routing mode
    pub fn icon(&self) -> &'static str {
        match self {
            RoutingMode::Bypass => "🔓",
            RoutingMode::Semantic => "🧠",
            RoutingMode::Hybrid => "⚡",
        }
    }
    
    /// Get description for routing mode
    pub fn description(&self) -> &'static str {
        match self {
            RoutingMode::Bypass => "Direct pass-through without LLM processing",
            RoutingMode::Semantic => "LLM predicts optimal routing for each request",
            RoutingMode::Hybrid => "Database rules with LLM fallback",
        }
    }
}

impl Default for RoutingMode {
    fn default() -> Self {
        RoutingMode::Hybrid
    }
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub mode: RoutingMode,
    pub confidence_threshold: f32,
    pub enable_learning: bool,
    pub fallback_to_bypass: bool,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            mode: RoutingMode::default(),
            confidence_threshold: 0.8,
            enable_learning: true,
            fallback_to_bypass: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_routing_mode_display() {
        assert_eq!(RoutingMode::Bypass.display_name(), "Bypass");
        assert_eq!(RoutingMode::Semantic.display_name(), "Semantic");
        assert_eq!(RoutingMode::Hybrid.display_name(), "Hybrid");
    }
    
    #[test]
    fn test_routing_mode_icons() {
        assert_eq!(RoutingMode::Bypass.icon(), "🔓");
        assert_eq!(RoutingMode::Semantic.icon(), "🧠");
        assert_eq!(RoutingMode::Hybrid.icon(), "⚡");
    }
}