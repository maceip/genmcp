# 🚀 Stage 1 Deliverable - Quick Start Guide

## Running the Multi-Transport MCP Proxy with Interactive TUI

### Prerequisites
- Rust toolchain installed
- Terminal with ANSI color support

### Build
```bash
cd ~/assist-mcp
cargo build --release
```

**Binary location:** `./target/release/mcp-cli`

### Usage Patterns

#### 1️⃣ **Monitor Only** (Start the TUI)
```bash
./target/release/mcp-cli monitor
# Or simply (monitor is the default):
./target/release/mcp-cli
```
This starts the interactive TUI on default socket `/tmp/mcp-monitor.sock`

#### 2️⃣ **Stdio Proxy** (Local MCP Server)
In a separate terminal:
```bash
./target/release/mcp-cli proxy \
  --command "npx -y @modelcontextprotocol/server-everything" \
  --name "local-mcp"
```
Note: `--transport stdio` is the default and can be omitted!

Or with a Python MCP server:
```bash
./target/release/mcp-cli proxy \
  --command "python mcp_server.py" \
  --name "python-server"
```

#### 3️⃣ **HTTP+SSE Proxy** (Remote MCP Server)
```bash
./target/release/mcp-cli proxy \
  --transport http-sse \
  --url "http://localhost:3000" \
  --name "remote-mcp"
```

#### 4️⃣ **Multiple Proxies** (Mix and Match!)
```bash
# Terminal 1: Monitor
./target/release/mcp-cli

# Terminal 2: Stdio proxy
./target/release/mcp-cli proxy \
  --command "npx -y @modelcontextprotocol/server-everything" \
  --name "local"

# Terminal 3: HTTP proxy
./target/release/mcp-cli proxy \
  --transport http-sse \
  --url "http://remote-mcp-server:3000" \
  --name "remote"
```

Both proxies appear in the same TUI! ✨

---

## 🎮 TUI Keyboard Controls

### Navigation
- `←/→` - Switch between Proxy List and Log View
- `↑/↓` - Scroll up/down
- `PgUp/PgDn` - Page up/down
- `Home/End` - Jump to top/bottom

### Tabs
- `Tab` - Next tab (All → Messages → Errors → System → All)
- `Shift+Tab` - Previous tab
- `1/2/3/4` - Jump directly to tab

### Search (Press `/`)
- Type to search logs and proxies with fuzzy matching
- `Enter` - Navigate to results
- `ESC` - Exit search
- Shows top 3 matches with scores and match reasons

### Actions
- `Enter` (on proxy) - Filter logs by that proxy
- `Enter` (on log) - Show detailed view
- `ESC` - Clear filter / Exit mode
- `c` - Clear all logs
- `r` - Refresh
- `?` - Show help dialog
- `q` or `Ctrl+C` - Quit

### Detail View (when viewing a log)
- `w` - Toggle word wrap
- `↑/↓` - Scroll
- `ESC` - Close detail view

---

## 🔍 Features Demonstrated

### Transport Indicators
- 📟 **stdio** - Local process
- 🌐 **HTTP+SSE** - Remote server
- 🔄 **HTTP Stream** - (Future)

### Fuzzy Search Engine
Press `/` to activate:
- **Jaro similarity** matching (typo-tolerant)
- Search by: message content, proxy name, log level
- **Visual score bars** (█████) showing match confidence
- **Match reasons**: Name match, Description match, Token match, Fuzzy match, Keyword match

### Real-time Monitoring
- Live log streaming with auto-scroll (Follow mode)
- Per-proxy stats: requests, errors, connections, bytes
- Tabbed filtering: All / Messages / Errors / System
- Proxy filtering: Click a proxy to see only its logs

---

## 📊 Example Session

```bash
# Terminal 1: Start monitor
./target/release/mcp-cli

# Terminal 2: Start a local MCP server
./target/release/mcp-cli proxy \
  --command "npx -y @modelcontextprotocol/server-everything" \
  --name "everything"
```

**In the TUI you'll see:**
1. Proxy appears in left panel: `🟢 📟 everything (0)`
2. Logs stream in right panel as server initializes
3. Press `2` to see only Messages (Request/Response)
4. Press `/` and type "init" to fuzzy search
5. Press `Enter` on a log to see JSON detail view
6. Press `ESC` twice to return to follow mode

---

## 🎯 Stage 1 Deliverable Verified

✅ **Proxy monitors stdio servers with interactive TUI**
✅ **Proxy monitors HTTP+SSE servers with interactive TUI**
✅ **Fuzzy search for logs and proxies**
✅ **Multi-transport support demonstrated**
✅ **Real-time stats and filtering working**

---

## 🐛 Troubleshooting

**"Connection refused" when starting proxy**
- Make sure monitor is running first
- Check socket path matches (default: `/tmp/mcp-monitor.sock`)

**No logs appearing**
- Verify MCP server is actually running (check with `ps aux | grep mcp`)
- Try verbose mode: `assist-mcp monitor --verbose`

**HTTP proxy fails**
- Verify HTTP server is reachable: `curl http://localhost:3000`
- Check URL format (must include scheme: `http://` or `https://`)

---

## 🎓 Advanced Usage

### Custom Socket Path
```bash
# Monitor
./target/release/mcp-cli monitor --ipc-socket /tmp/custom.sock

# Proxy
./target/release/mcp-cli proxy \
  --ipc-socket /tmp/custom.sock \
  --command "..." \
  --name "..."
```

### Verbose Logging
```bash
./target/release/mcp-cli monitor --verbose
# Logs written to /tmp/mcp-monitor.log
```

### Shell Commands (stdio transport)
```bash
# Shell is enabled by default, use --shell false to disable
./target/release/mcp-cli proxy \
  --command "cd /path/to/server && python server.py" \
  --name "complex-server"
```

### API Key for HTTP Servers
```bash
./target/release/mcp-cli proxy \
  --transport http-sse \
  --url "https://api.example.com" \
  --api-key "your-api-key-here" \
  --name "authenticated-server"
```

Note: API key support is CLI-ready but pending mcp-core integration.
