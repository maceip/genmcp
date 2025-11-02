use anyhow::{anyhow, Result};
use mcp_common::TransportType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportConfig {
    Stdio {
        command: String,
        use_shell: bool,
    },
    HttpSse {
        url: String,
        api_key: Option<String>,
    },
    HttpStream {
        url: String,
        api_key: Option<String>,
    },
}

impl TransportConfig {
    pub fn transport_type(&self) -> TransportType {
        match self {
            TransportConfig::Stdio { .. } => TransportType::Stdio,
            TransportConfig::HttpSse { .. } => TransportType::HttpSse,
            TransportConfig::HttpStream { .. } => TransportType::HttpStream,
        }
    }

    pub fn display_target(&self) -> String {
        match self {
            TransportConfig::Stdio { command, .. } => command.clone(),
            TransportConfig::HttpSse { url, .. } => url.clone(),
            TransportConfig::HttpStream { url, .. } => url.clone(),
        }
    }

    pub fn from_cli_args(
        transport: &str,
        command: Option<String>,
        url: Option<String>,
        use_shell: bool,
        api_key: Option<String>,
    ) -> Result<Self> {
        match transport {
            "stdio" => {
                let command = command.ok_or_else(|| {
                    anyhow!("--command is required for stdio transport")
                })?;
                Ok(TransportConfig::Stdio { command, use_shell })
            }
            "http-sse" => {
                let url = url.ok_or_else(|| {
                    anyhow!("--url is required for http-sse transport")
                })?;
                Ok(TransportConfig::HttpSse { url, api_key })
            }
            "http-stream" => {
                let url = url.ok_or_else(|| {
                    anyhow!("--url is required for http-stream transport")
                })?;
                Ok(TransportConfig::HttpStream { url, api_key })
            }
            _ => Err(anyhow!(
                "Invalid transport type: {}. Must be one of: stdio, http-sse, http-stream",
                transport
            )),
        }
    }
}
