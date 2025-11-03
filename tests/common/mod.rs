//! Common test utilities shared across integration and E2E tests

pub mod test_server;
pub mod helpers;
pub mod python_server;

// Re-export commonly used items
pub use test_server::*;
pub use helpers::*;
pub use python_server::*;

/// Common test configuration
pub struct TestConfig {
    pub test_port: u16,
    pub temp_dir: String,
    pub log_level: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_port: 8080,
            temp_dir: std::env::temp_dir().to_string_lossy().to_string(),
            log_level: "debug".to_string(),
        }
    }
}

/// Setup logging for tests
pub fn setup_test_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .init();
}

/// Create a temporary directory for tests
pub fn create_temp_dir() -> std::path::PathBuf {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    temp_dir.into_path()
}
