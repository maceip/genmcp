use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "assist-mcp")]
#[command(about = "Intelligent MCP proxy with monitoring")]
#[command(version = "0.2.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the MCP monitor (default if no subcommand provided)
    Monitor {
        /// IPC socket path for proxy communication
        #[arg(short, long, default_value = "/tmp/mcp-monitor.sock")]
        ipc_socket: String,

        /// Verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Start an MCP proxy server
    Proxy {
        /// Transport type (stdio, http-sse, http-stream)
        #[arg(short, long, default_value = "stdio")]
        transport: String,

        /// MCP server command (for stdio transport)
        #[arg(short, long)]
        command: Option<String>,

        /// HTTP URL (for http-sse or http-stream transport)
        #[arg(short, long)]
        url: Option<String>,

        /// API key for HTTP transports
        #[arg(long)]
        api_key: Option<String>,

        /// Name for this proxy instance
        #[arg(short, long, default_value = "mcp-transport")]
        name: String,

        /// IPC socket path for monitor communication
        #[arg(short = 'i', long, default_value = "/tmp/mcp-monitor.sock")]
        ipc_socket: String,

        /// Verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Use shell to execute command (enabled by default for stdio)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        shell: bool,

        /// Skip connecting to monitor (standalone mode)
        #[arg(long, default_value_t = false)]
        no_monitor: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Monitor {
            ipc_socket,
            verbose,
        }) => run_monitor(ipc_socket, verbose).await,
        Some(Commands::Proxy {
            transport,
            command,
            url,
            api_key,
            name,
            ipc_socket,
            verbose,
            shell,
            no_monitor,
        }) => run_proxy(transport, command, url, api_key, name, ipc_socket, verbose, shell, no_monitor).await,
        None => {
            // Default to monitor
            run_monitor("/tmp/mcp-monitor.sock".to_string(), false).await
        }
    }
}

async fn run_monitor(ipc_socket: String, verbose: bool) -> Result<()> {
    // Import the monitor functionality
    use mcp_ui::{run_monitor_app, MonitorArgs};

    let args = MonitorArgs {
        ipc_socket,
        verbose,
    };

    run_monitor_app(args).await
}

async fn run_proxy(
    transport: String,
    command: Option<String>,
    url: Option<String>,
    api_key: Option<String>,
    name: String,
    ipc_socket: String,
    verbose: bool,
    shell: bool,
    no_monitor: bool,
) -> Result<()> {
    // Import the proxy functionality
    use mcp_transport::{run_proxy_app, ProxyArgs, TransportConfig};

    // Build transport config from CLI args
    let transport_config = TransportConfig::from_cli_args(
        &transport,
        command,
        url,
        shell,
        api_key,
    )?;

    let args = ProxyArgs {
        transport_config,
        name,
        ipc_socket,
        verbose,
        no_monitor,
    };

    run_proxy_app(args).await
}
