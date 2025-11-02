# Stage 1: Remaining Tasks

## Completed ✅
- [x] Merge mcp-trace + mcp-probe-core foundations
- [x] Unified workspace with 183 tests passing
- [x] Renamed crates for clarity:
  - mcp-trace → mcp-cli
  - mcp-monitor → mcp-ui
  - mcp-proxy → mcp-transport

## Remaining for Stage 1 Completion

### Task 1: Integrate HTTP+SSE Transport into mcp-transport
**Goal:** Enable mcp-transport to connect to HTTP+SSE servers, not just stdio

**Current State:**
- mcp-transport only supports stdio (via stdio_handler.rs)
- mcp-core has full HTTP+SSE transport implementation
- Need to wire them together

**Implementation Steps:**
1. Add mcp-core dependency to mcp-transport/Cargo.toml
2. Create new transport abstraction in mcp-transport
3. Update CLI to support --transport flag (stdio|http-sse|http-stream)
4. Update IPC messages to include transport type
5. Update mcp-ui to show transport type in proxy list
6. Test with both stdio and HTTP+SSE servers

**Files to Modify:**
- `mcp-transport/Cargo.toml` - Add mcp-core dependency
- `mcp-transport/src/lib.rs` - Export transport types
- `mcp-transport/src/main.rs` - Add --transport CLI flag
- `mcp-transport/src/proxy.rs` - Abstract transport handling
- `mcp-common/src/types.rs` - Add TransportType enum
- `mcp-ui/src/ui.rs` - Display transport type

**Success Criteria:**
- Can start proxy with: `mcp-cli proxy --transport http-sse --url http://localhost:3000`
- Can start proxy with: `mcp-cli proxy --transport stdio --command "python server.py"`
- UI shows transport type for each proxy
- Both transport types work simultaneously

---

### Task 2: Add Fuzzy Search to mcp-ui
**Goal:** Enable searching across proxies, logs, and methods in the TUI

**Current State:**
- mcp-probe has excellent search.rs implementation
- mcp-ui has no search functionality
- Need to port fuzzy search to TUI

**Implementation Steps:**
1. Copy search.rs from mcp-probe CLI to mcp-ui
2. Add search state to App struct
3. Add search bar to UI (activated with '/' key)
4. Implement search across:
   - Proxy names
   - Log messages
   - MCP method names (tools/call, resources/read, etc.)
5. Show search results with highlighting
6. Add keyboard navigation for results

**Files to Modify:**
- `mcp-ui/src/search.rs` - New file from mcp-probe
- `mcp-ui/src/app.rs` - Add search state and logic
- `mcp-ui/src/ui.rs` - Add search bar widget
- `mcp-ui/src/lib.rs` - Export search module

**Success Criteria:**
- Press '/' to activate search
- Type to filter logs in real-time
- ESC to clear search
- Enter to lock search results
- Shows match count

---

### Task 3: Test Multi-Transport Scenario
**Goal:** Verify everything works together

**Test Scenario:**
```bash
# Terminal 1: Start UI
cargo run -p mcp-cli monitor

# Terminal 2: Start stdio proxy
cargo run -p mcp-cli proxy \
  --transport stdio \
  --command "python test-mcp-server/test_server.py" \
  --name "stdio-server"

# Terminal 3: Start HTTP+SSE proxy (if available)
cargo run -p mcp-cli proxy \
  --transport http-sse \
  --url "http://localhost:3000" \
  --name "http-server"
```

**Success Criteria:**
- Both proxies show up in UI
- Transport types are visible
- Can search across both
- Logs stream correctly from both
- No crashes or errors

---

## Estimated Time

- Task 1 (HTTP+SSE integration): **3-4 hours**
- Task 2 (Fuzzy search): **2-3 hours**
- Task 3 (Testing): **1 hour**

**Total: 6-8 hours** (rest of today + tomorrow morning)

---

## Next Steps

Let's start with **Task 1: HTTP+SSE Transport Integration** since it's the most critical feature.

Would you like to:
1. Start implementing Task 1 now?
2. Review the plan first?
3. Adjust priorities?
