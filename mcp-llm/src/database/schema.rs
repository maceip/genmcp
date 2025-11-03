//! Database schema and main database struct

use sqlx::SqlitePool;
use crate::error::{LlmError, LlmResult};
use super::{RoutingRulesDatabase, PredictionsDatabase, MetricsDatabase};

/// Main LLM database coordinator
pub struct LlmDatabase {
    pub pool: SqlitePool,
    pub routing_rules: RoutingRulesDatabase,
    pub predictions: PredictionsDatabase,
    pub metrics: MetricsDatabase,
}

impl LlmDatabase {
    /// Create new database instance
    pub async fn new(database_url: &str) -> LlmResult<Self> {
        let pool = sqlx::SqlitePool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self {
            routing_rules: RoutingRulesDatabase::new(pool.clone()),
            predictions: PredictionsDatabase::new(pool.clone()),
            metrics: MetricsDatabase::new(pool.clone()),
            pool,
        })
    }
}

impl Clone for LlmDatabase {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            routing_rules: self.routing_rules.clone(),
            predictions: self.predictions.clone(),
            metrics: self.metrics.clone(),
        }
    }
}