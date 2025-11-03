<div align="center">
<img width="320" height="229" alt="GEM2" src="https://github.com/user-attachments/assets/cdc7c62a-7890-4f97-98d1-35525500b0e9" />
</div>



## 🟡

`MCP is a pain to configure, discover, and use the way you want`

- Complex setup with multiple configuration files and environment variables
- No visibility into which tools are available or how they're being used
- Every request goes through the same path regardless of complexity
- No way to optimize tool selection or predict which tools will be needed
- Difficult to monitor and debug tool usage in real-time
- Sandboxing untrusted code requires infrastructure



## its a client-side thing
<img height="266" alt="Screenshot 2025-11-02 at 5 55 34 AM" src="https://github.com/user-attachments/assets/24182435-d1e3-4500-a2e8-9f4703e07be7" />

**genmcp** brings the fun back in for MCP: route, optimize, generate [securely]

Stack:
- fast: async runtime with tokio & sql in WAL mode for routing decisions (0.01-0.05ms)
- universal: openai-api, rust lib, mcp
- alpha: dspy-rs based GEPA optimizer for optimized tool aggregation; spend less faster
- standard: http, sse, stdio, ws
- secure: litert-lm serves, your prompts trigger new mcp tools, our sandbox saves

## Features

- [x] Proxy stdio
- [x] Proxy SSE
- [x] Proxy HTTP
- [x] Accelerated LLM
- [x] Terminal UI
- [ ] Generative Tool Injection
- [x] Sandbox runtime (WASM/WASI with wasmtime) and custom bochs transpiled linux env

## Quick Start

```bash
# Build
cargo build --release

# Run server
cargo run --release

# Run with custom config
DATABASE_PATH=./proxy.db \
UPSTREAM_URL=http://localhost:9000 \
HTTP_LISTEN_ADDR=127.0.0.1:4000 \
cargo run --release

# Examples
cargo run --example llm_demo
cargo run --example gepa_tool_judge
cargo run --example wasm_test
```

## Configuration

Environment variables:

```bash
DATABASE_PATH=./mcp_proxy.db               # SQLite database file
UPSTREAM_URL=http://localhost:9000         # Upstream MCP server
MODEL_NAME=gemma-3n-E4B                    # LLM model name
UI_LISTEN_ADDR=127.0.0.1:8081             # WebSocket UI address
HTTP_LISTEN_ADDR=127.0.0.1:4000           # HTTP transport address
ENABLE_STDIO=false                         # Enable stdio transport
LM_BASE_URL=http://localhost:3000/v1      # LLM API base URL
LM_API_KEY=                                # LLM API key (optional)
LM_TEMPERATURE=0.7                         # LLM temperature
LM_MAX_TOKENS=512                          # LLM max tokens
WASM_PATH=$HOME/amd64.wasm                 # Path to WASM module (for examples)
```

## Architecture

### Dual-Path Routing

1. **Fast Path (Bypass)**: Direct forwarding to upstream server
   - No LLM overhead
   - Sub-100ms latency (network bound)
   - For simple, predictable tools

2. **Slow Path (Semantic)**: LLM-enhanced routing
   - Tool selection based on context
   - Request modification
   - Prediction accuracy tracking
   - For complex, context-dependent tools

### Routing Rules

SQLite database stores per-tool routing decisions:

```sql
-- View routing rules
SELECT * FROM tool_rules;

-- Add bypass rule
INSERT INTO tool_rules (tool_name, should_route) VALUES ('simple_get', 0);

-- Add semantic routing rule
INSERT INTO tool_rules (tool_name, should_route) VALUES ('complex_analysis', 1);

-- Update rule
UPDATE tool_rules SET should_route = 1 WHERE tool_name = 'simple_get';
```

### GEPA Optimization

Generated Expert Performance Analyzer improves tool prediction:

- dspy-rs for structured prediction optimization
- Feedback loop from actual tool usage
- Learning based on success/failure rates
- Automatic prompt refinement

### Sandbox Runtime

WASM-based sandboxing for untrusted code:

- wasmtime runtime with WASI support
- Capability-based security with cap-std
- Memory-safe execution
- Python, JavaScript, and other WASM-compiled languages

## Monitoring

### WebSocket UI

Access at ws://127.0.0.1:8081/ws

Shows request/response pairs, routing decisions, LLM predictions, and latency statistics.

### Terminal UI

```bash
cargo run --release --features tui
```

## Performance

- Database lookup: 0.01-0.05ms
- Fast path: 10-100ms (network to upstream)
- Memory: ~10MB + SQLite cache
- Concurrent requests: thousands

## Resilience

- Database failure → automatic bypass mode
- LLM service down → automatic bypass
- Upstream failure → error returned to client
- SQLite WAL mode → concurrent reads during writes

## License

MIT
