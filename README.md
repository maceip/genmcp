# Assist MCP

**Intelligent MCP Proxy with Real-Time Monitoring**

An advanced Model Context Protocol (MCP) proxy that combines transparent monitoring, traffic modification, and LLM-powered intelligent routing.

## Features

### Stage 1 - Foundation ✅ COMPLETE
- ✅ **Multi-transport support** - stdio, HTTP+SSE (HTTP streaming planned)
- ✅ **Real-time TUI monitoring** - Monitor multiple MCP servers simultaneously
- ✅ **Transparent interception** - Zero-impact STDIO proxying
- ✅ **Resilient IPC** - Buffered communication with auto-reconnection
- ✅ **Complete protocol support** - Tools, resources, prompts, logging, sampling
- ✅ **Fuzzy search** - Fast keyword matching with similarity scoring
- ✅ **122 passing tests** - Comprehensive test coverage

### Stage 2 - Traffic Modification ✅ COMPLETE
- ✅ **Interceptor framework** - Pluggable message interceptors with stats tracking
- ✅ **4 Built-in interceptors** - Logging, validation, rate limiting, transform
- ✅ **Request/response transformation** - JSON path-based field modifications
- ✅ **Hooks TUI tab** - Real-time interceptor statistics and monitoring
- ✅ **22 interceptor tests** - Full test coverage for all interceptor types

### Stage 3 - LLM Intelligence 🔨 IN PROGRESS
- 🔨 **mcp-llm crate** - LiteRT-LM integration with C++ bindings (not yet integrated)
- 🔨 **DSPy-RS** - Structured prediction and tool routing (implemented)
- 🔨 **SQLite database** - Routing rules and metrics storage (implemented)
- 🔨 **GEPA optimizer** - Prompt optimization framework (implemented)
- ⏳ **Integration** - Connect mcp-llm to main proxy workflow (pending)

## Quick Start

```bash
# Build all components
cargo build --release

# Terminal 1: Start UI monitor
./target/release/mcp-cli monitor

# Terminal 2: Start transport proxy with stdio server
./target/release/mcp-cli proxy \
  --name "My Server" \
  --command "python server.py"
```

## Architecture

```
Client → [mcp-transport] ────→ MCP Server
         │  ↓ Interceptors    (stdio/HTTP+SSE)
         │  • Logging
         │  • Validation
         │  • Rate Limiting
         │  • Transform
         │
         └──→ BufferedIPC ──→ [mcp-ui TUI]
              (Unix socket)   • 5 tabs (All/Messages/Errors/System/Hooks)
                             • Fuzzy search
                             • Multi-proxy monitoring
                             • Interceptor statistics
```

**Transport Features:**
- Intercepts STDIO communication with zero overhead
- Multiple transport types (stdio ✅, HTTP+SSE ✅, HTTP streaming ⏳)
- Pluggable interceptor framework for traffic modification
- Sends logs + stats to UI via Unix socket IPC
- Resilient buffered IPC (works offline, auto-reconnects)
- Per-message interceptor overhead <1ms

**UI Features:**
- Real-time log streaming with [MODIFIED] indicators
- Multi-proxy support with transport indicators (📟 stdio, 🌐 HTTP+SSE)
- 5 tabs: All, Messages, Errors, System, **Hooks** (interceptor stats)
- Fuzzy search with keyword matching (press `/` to activate)
- Proxy selection and detail views with word wrap
- Statistics dashboard with performance metrics
- Keyboard shortcuts: `1-5` (jump to tab), `h` (help), `w` (word wrap)

## Project Structure

- `mcp-core/` - MCP protocol types, transports, and interceptor framework
- `mcp-common/` - IPC communication and shared types
- `mcp-transport/` - Transport layer proxy with interceptors (stdio, HTTP+SSE)
- `mcp-ui/` - TUI monitoring application with fuzzy search and hooks tab
- `mcp-cli/` - Unified CLI binary (`monitor`, `proxy` commands)
- `mcp-llm/` - LLM integration (LiteRT-LM, DSPy-RS, GEPA, SQLite) - **not yet integrated**
- `tests/` - End-to-end integration tests

**Note:** `mcp-llm` contains a complete Stage 3 implementation but is not yet connected to the main proxy workflow.

## Development

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p mcp-core
cargo test -p mcp-common
cargo test -p mcp-transport
cargo test -p mcp-ui

# Run E2E tests
cargo test --test e2e_tests

# Check compilation
cargo check --workspace
```

## Testing

**122 tests across:**
- Protocol implementation (mcp-core: 93 tests)
- IPC communication (mcp-common)
- Transport layer (mcp-transport: 17 tests)
- Interceptors (mcp-transport: 22 tests including integration)
- TUI application (mcp-ui: 4 tests)
- End-to-end scenarios (tests/: 3 tests)

```bash
# Run all tests
cargo test --workspace --lib

# Run with coverage
cargo test --workspace --lib -- --nocapture
```

## Roadmap

### Stage 1: Foundation ✅ COMPLETE
- [x] Merge mcp-trace + mcp-probe-core
- [x] Unified workspace with 122 tests
- [x] Integrate HTTP+SSE transport into proxy
- [x] Add fuzzy search to TUI

### Stage 2: Traffic Modification ✅ COMPLETE
- [x] Message interceptor framework (InterceptorManager)
- [x] Built-in interceptors (logging, validation, rate limiting, transform)
- [x] Request/response transformation (JSON path rules)
- [x] Hooks tab in TUI with real-time stats

### Stage 3: LLM Intelligence 🔨 IN PROGRESS
- [x] Tool prediction with dspy-rs (implemented in mcp-llm)
- [x] SQLite-backed routing decisions (implemented in mcp-llm)
- [x] GEPA optimizer for continuous improvement (implemented in mcp-llm)
- [ ] **Integrate mcp-llm into main workflow** ⏳ NEXT
- [ ] Prediction accuracy metrics in TUI
- [ ] Real-time routing decision visualization

**Current Focus:** Integrating the completed mcp-llm crate into the proxy pipeline

## Credits

Built by combining:
- **[mcp-trace](https://github.com/zabirauf/mcp-trace)** - Monitoring and TUI foundation
- **[mcp-probe](https://github.com/conikeec/mcp-probe)** - Protocol and transport implementation

## License

MIT (inherits from source projects)
