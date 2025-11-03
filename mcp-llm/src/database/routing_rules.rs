//! Database operations for routing rules

use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::{LlmError, LlmResult};

#[derive(Debug, Clone)]
pub struct RoutingRulesDatabase {
    pool: SqlitePool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoutingRule {
    pub id: String,
    pub pattern: String,
    pub target_tool: String,
    pub target_transport: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled: bool,
}

impl RoutingRulesDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Create new routing rule
    pub async fn create_rule(
        &self,
        pattern: &str,
        target_tool: &str,
        target_transport: &str,
        confidence: f64,
    ) -> LlmResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        sqlx::query!(
            "INSERT INTO routing_rules (id, pattern, target_tool, target_transport, confidence, created_at, updated_at, enabled) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            id,
            pattern,
            target_tool,
            target_transport,
            confidence,
            now,
            now,
            true
        )
        .execute(&self.pool)
        .await?;
        
        Ok(id)
    }
    
    /// Get routing rule by pattern
    pub async fn get_rule_by_pattern(&self, pattern: &str) -> LlmResult<Option<RoutingRule>> {
        let rule = sqlx::query_as!(
            RoutingRule,
            "SELECT * FROM routing_rules WHERE pattern = ? AND enabled = true",
            pattern
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(rule)
    }
    
    /// Find matching routing rule for request
    pub async fn find_matching_rule(&self, request_content: &str) -> LlmResult<Option<RoutingRule>> {
        let rules = sqlx::query_as!(
            RoutingRule,
            "SELECT * FROM routing_rules WHERE enabled = true ORDER BY confidence DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Simple pattern matching (could be enhanced with regex)
        for rule in rules {
            if request_content.contains(&rule.pattern) {
                return Ok(Some(rule));
            }
        }
        
        Ok(None)
    }
    
    /// Update rule confidence based on feedback
    pub async fn update_rule_confidence(&self, rule_id: &str, new_confidence: f64) -> LlmResult<()> {
        let now = Utc::now();
        
        sqlx::query!(
            "UPDATE routing_rules SET confidence = ?, updated_at = ? WHERE id = ?",
            new_confidence,
            now,
            rule_id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// List all enabled rules
    pub async fn list_enabled_rules(&self) -> LlmResult<Vec<RoutingRule>> {
        let rules = sqlx::query_as!(
            RoutingRule,
            "SELECT * FROM routing_rules WHERE enabled = true ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rules)
    }
}

impl Clone for RoutingRulesDatabase {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}