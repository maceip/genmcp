# Assist MCP

**Intelligent MCP Proxy with Real-Time Monitoring**

An advanced Model Context Protocol (MCP) proxy that combines transparent monitoring, traffic modification, and LLM-powered intelligent routing.

## Features

### Current (Stage 1 - Foundation)
- ✅ **Multi-transport support** - stdio, HTTP+SSE, HTTP streaming
- ✅ **Real-time TUI monitoring** - Monitor multiple MCP servers simultaneously
- ✅ **Transparent interception** - Zero-impact STDIO proxying
- ✅ **Resilient IPC** - Buffered communication with auto-reconnection
- ✅ **Complete protocol support** - Tools, resources, prompts, logging, sampling
- ✅ **183 passing tests** - Comprehensive test coverage

### Coming Soon
- 🔄 **Stage 2: Traffic Modification** - Interceptor hooks for request/response modification
- 🤖 **Stage 3: LLM Intelligence** - AI-powered routing, tool prediction, GEPA optimization

## Quick Start

```bash
# Build all components
cargo build --release

# Terminal 1: Start monitor
./target/release/mcp-trace monitor

# Terminal 2: Start proxy with stdio server
./target/release/mcp-trace proxy \
  --name "My Server" \
  --command "python server.py"
```

## Architecture

```
Client → [mcp-proxy] → MCP Server
             ↓ (IPC)
        [mcp-monitor TUI]
```

**Proxy Features:**
- Intercepts STDIO communication
- Multiple transport types (stdio, HTTP+SSE, HTTP streaming)
- Sends logs to monitor via Unix socket IPC
- Resilient buffered IPC (works offline)

**Monitor Features:**
- Real-time log streaming
- Multi-proxy support
- Tab-based filtering (All, Messages, Errors, System)
- Proxy selection and detail views
- Statistics dashboard

## Project Structure

- `mcp-core/` - MCP protocol types and transports (from mcp-probe)
- `mcp-common/` - IPC communication and shared types (from mcp-trace)
- `mcp-proxy/` - STDIO proxy implementation (from mcp-trace)
- `mcp-monitor/` - TUI monitoring application (from mcp-trace)
- `mcp-trace/` - Unified CLI binary
- `tests/` - End-to-end integration tests

## Development

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p mcp-core
cargo test -p mcp-common
cargo test -p mcp-proxy
cargo test -p mcp-monitor

# Run E2E tests
cargo test --test e2e_tests

# Check compilation
cargo check --workspace
```

## Testing

**183 tests across:**
- Protocol implementation (mcp-core)
- IPC communication (mcp-common)
- Proxy logic (mcp-proxy)
- TUI application (mcp-monitor)
- End-to-end scenarios (tests/)

## Roadmap

### Stage 1: Foundation (CURRENT)
- [x] Merge mcp-trace + mcp-probe-core
- [x] Unified workspace with 183 tests
- [ ] Integrate HTTP+SSE transport into proxy
- [ ] Add fuzzy search to TUI

### Stage 2: Traffic Modification (Next)
- [ ] Message interceptor framework
- [ ] Built-in interceptors (logging, validation, rate limiting)
- [ ] Request/response transformation
- [ ] Interactive hook management in TUI

### Stage 3: LLM Intelligence (Future)
- [ ] Tool prediction with dspy-rs
- [ ] SQLite-backed routing decisions
- [ ] GEPA optimizer for continuous improvement
- [ ] Prediction accuracy metrics in TUI

## Credits

Built by combining:
- **[mcp-trace](https://github.com/zabirauf/mcp-trace)** - Monitoring and TUI foundation
- **[mcp-probe](https://github.com/conikeec/mcp-probe)** - Protocol and transport implementation

## License

MIT (inherits from source projects)
