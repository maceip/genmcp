# Update tests/lib.rs
cat > /Users/rpm/assist-mcp/tests/lib.rs << 'EOF'
//! Integration and E2E tests for assist-mcp

pub mod common;
pub mod e2e;
pub mod integration;
EOF

# Update tests/Cargo.toml
cat > /Users/rpm/assist-mcp/tests/Cargo.toml << 'EOF'
[package]
name = "assist-mcp-tests"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core dependencies
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

# Test utilities
tokio-test = "0.4"
tempfile = "3.8"
wiremock = "0.6"
assert_matches = "1.5"
tracing-test = "0.2"

# Internal dependencies
mcp-common = { path = "../mcp-common" }
mcp-core = { path = "../mcp-core" }
mcp-transport = { path = "../mcp-transport" }
mcp-llm = { path = "../mcp-llm" }

[[bin]]
name = "test-runner"
path = "src/main.rs"
EOF
