use thiserror::Error;

/// Detailed error types for LLM integration
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("LiteRT-LM binding error: {0}")]
    BindingError(String),

    #[error("LiteRT-LM runtime error: {0}")]
    RuntimeError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

pub type LlmResult<T> = Result<T, LlmError>;

/// Convert from C++ string errors
impl From<String> for LlmError {
    fn from(s: String) -> Self {
        LlmError::RuntimeError(s)
    }
}