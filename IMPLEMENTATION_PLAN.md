# Assist MCP - Implementation Plan

## Project Goal
Combine mcp-trace (monitoring) + mcp-probe (protocol/transports) into a unified intelligent MCP proxy with TUI.

## Stage 1: Foundation Merge (CURRENT)
**Duration:** 2-3 days
**Status:** In Progress

### Completed
- ✅ Created ~/assist-mcp repository
- ✅ Copied mcp-trace as foundation (proxy, monitor, common, tests)
- ✅ Imported mcp-probe-core (transport layer)
- ✅ Updated workspace Cargo.toml with unified dependencies
- ✅ Verified workspace compiles

### In Progress
- [ ] Integrate mcp-core transports into mcp-proxy
  - [ ] Add HTTP+SSE transport alongside stdio
  - [ ] Update CLI to support transport selection
  - [ ] Update IPC messages to include transport info
- [ ] Add fuzzy search to mcp-monitor TUI
  - [ ] Copy search.rs from mcp-probe
  - [ ] Add search bar to TUI
  - [ ] Search across proxies, logs, tools
- [ ] Run all tests and fix integration issues

### Success Criteria
- Proxy can connect to both stdio and HTTP+SSE servers
- Monitor TUI has fuzzy search capability
- All 102+ tests pass
- Can monitor multiple servers simultaneously

---

## Stage 2: Traffic Modification Hooks
**Duration:** 2-3 days
**Status:** COMPLETE ✅

### Goals
- Import mcp-core's MessageInterceptor framework ✅
- Add interceptor chain to proxy ✅
- Build basic interceptors (logging, validation, rate limit) ✅
- Show interceptor status in TUI (messages marked as [MODIFIED]) ✅

### Completed Tasks
- ✅ Used mcp-core's existing interceptor.rs framework
- ✅ Added InterceptorManager to StdioHandler
- ✅ Implemented built-in interceptors:
  - ✅ LoggingInterceptor - logs all MCP traffic for debugging
  - ✅ ValidationInterceptor - validates JSON-RPC protocol compliance (strict/lenient mode)
  - ✅ RateLimitInterceptor - sliding window rate limiting per method (permissive/moderate/strict presets)
- ✅ Updated StdioHandler to process messages through interceptor chain
- ✅ Added [MODIFIED] indicator in traffic logs
- ✅ Wrote comprehensive tests:
  - 12 unit tests for individual interceptors
  - 5 integration tests for InterceptorManager
  - All tests passing

### Success Criteria Achieved
- ✅ Interceptors can modify traffic transparently
- ✅ Logs show which messages were modified with [MODIFIED] prefix
- ✅ Interceptors can be added/configured via InterceptorManager
- ✅ Rate limiting works correctly with sliding window algorithm
- ✅ Validation interceptor can block invalid messages in strict mode
- ✅ Messages can be blocked by interceptors (won't be forwarded)

### Architecture Implemented
```rust
User Input
    ↓
StdioHandler::process_outgoing()
    ↓
InterceptorManager::process_message()
    ↓
[LoggingInterceptor (priority 10)]
    ↓
[ValidationInterceptor (priority 20)]
    ↓
[RateLimitInterceptor (priority 30)]
    ↓
Modified/Blocked/Passed Through
    ↓
Forward to MCP Server (or block)
```

### Test Results
```
✅ 12 interceptor unit tests passing:
   - LoggingInterceptor: 2 tests
   - ValidationInterceptor: 5 tests
   - RateLimitInterceptor: 5 tests

✅ 5 integration tests passing:
   - InterceptorManager registration
   - Priority ordering
   - Message blocking
   - Rate limiting
   - Stats tracking
```

---

## Stage 3: LLM Integration
**Duration:** 3-4 days
**Status:** Not Started

### Goals
- Import LLM predictor from routemcp-backup
- Add intelligent routing with SQLite
- Implement GEPA optimizer
- Show predictions in TUI

### Tasks
- [ ] Create mcp-llm crate
- [ ] Copy llm_predictor.rs from routemcp
- [ ] Copy lm_provider.rs (dspy-rs integration)
- [ ] Copy GEPA optimizer
- [ ] Add SQLite database for routing rules
- [ ] Implement LlmInterceptor
- [ ] Add LLM panel to TUI
- [ ] Show prediction accuracy metrics
- [ ] Add routing mode toggle (bypass/semantic/hybrid)

### Success Criteria
- Can predict which tool will be called
- Records prediction accuracy
- Routes based on database rules
- GEPA improves predictions over time
- TUI shows live accuracy stats

---

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│  Client (Claude Desktop, Cursor, etc.)     │
└──────────────────┬──────────────────────────┘
                   ↓
┌─────────────────────────────────────────────┐
│  mcp-proxy                                  │
│  ┌─────────────────────────────────────┐   │
│  │ Transport Layer (mcp-core)          │   │
│  │  • stdio                            │   │
│  │  • HTTP+SSE                         │   │
│  │  • HTTP streaming                   │   │
│  └──────────────┬──────────────────────┘   │
│                 ↓                           │
│  ┌─────────────────────────────────────┐   │
│  │ Interceptor Chain                   │   │
│  │  • Logging                          │   │
│  │  • Validation                       │   │
│  │  • Rate Limiting                    │   │
│  │  • LLM Enhancement (Stage 3)        │   │
│  └──────────────┬──────────────────────┘   │
│                 ↓                           │
│  Forward to MCP Server                     │
└──────────────┬──────────────────────────────┘
               ↓                      │
         MCP Server                   │ (IPC)
               ↓                      ↓
┌─────────────────────────────────────────────┐
│  mcp-monitor TUI                            │
│  ┌──────────┬─────────────┬──────────────┐ │
│  │ Proxies  │   Traffic   │   Search     │ │
│  │ (stdio,  │   (with     │   (fuzzy)    │ │
│  │  HTTP)   │   modified) │              │ │
│  └──────────┴─────────────┴──────────────┘ │
└─────────────────────────────────────────────┘
```

---

## Dependencies Added

### From mcp-probe
- reqwest 0.12 (HTTP client with streaming)
- eventsource-stream 0.2 (SSE parsing)
- url 2.5 (URL parsing)
- bytes 1.5 (Byte buffers)
- futures 0.3 (Future combinators)
- async-trait 0.1 (Async trait support)

### For Stage 3 (Future)
- dspy-rs 0.7.1 (LLM orchestration)
- sqlx 0.8 (SQLite for routing rules)
- wasmtime 35.0 (WASM sandbox - optional)

---

## Testing Strategy

### Stage 1
- Run mcp-trace's 102 existing tests
- Add tests for HTTP+SSE transport
- Add tests for search functionality
- E2E test with both stdio and HTTP servers

### Stage 2
- Unit tests for each interceptor
- Integration tests for interceptor chain
- Test priority ordering
- Test modification correctness

### Stage 3
- LLM prediction accuracy tests
- Database routing tests
- GEPA optimizer tests
- E2E with real LLM service

---

## Current Focus: Stage 1 - Transport Integration

Next steps:
1. Review mcp-proxy/src/stdio_handler.rs
2. Abstract transport layer to support multiple types
3. Add HTTP+SSE support using mcp-core
4. Update CLI to select transport type
5. Test with both stdio and HTTP+SSE servers
