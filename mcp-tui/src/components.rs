use chrono::{DateTime, Utc};
use ratatui::style::{Color, Style};

pub use crate::activity_feed::ActivityFeed;
pub use crate::clients_panel::ClientsPanel;
pub use crate::query_input::QueryInput;
pub use crate::quick_access::{QuickAccess, QuickAction};
pub use crate::servers_panel::ServersPanel;

/// Identifies which widget currently owns input focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Clients,
    Servers,
    Activity,
    QuickAccess,
    QueryInput,
}

/// Connection status for a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    Connected,
    Disconnected,
    Error,
}

impl ClientStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Connected => "Connected",
            Self::Disconnected => "Disconnected",
            Self::Error => "Error",
        }
    }

    pub fn style(&self) -> Style {
        match self {
            Self::Connected => Style::default().fg(Color::Green),
            Self::Disconnected => Style::default().fg(Color::DarkGray),
            Self::Error => Style::default().fg(Color::Red),
        }
    }
}

/// Status for a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStatus {
    Starting,
    Running,
    Degraded,
    Stopped,
    Error,
}

impl ServerStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Starting => "Starting",
            Self::Running => "Running",
            Self::Degraded => "Degraded",
            Self::Stopped => "Stopped",
            Self::Error => "Error",
        }
    }

    pub fn style(&self) -> Style {
        match self {
            Self::Starting => Style::default().fg(Color::Yellow),
            Self::Running => Style::default().fg(Color::Green),
            Self::Degraded => Style::default().fg(Color::LightYellow),
            Self::Stopped => Style::default().fg(Color::DarkGray),
            Self::Error => Style::default().fg(Color::Red),
        }
    }
}

/// Activity execution status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityStatus {
    Processing,
    Success,
    Failed,
}

impl ActivityStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Processing => "Processing",
            Self::Success => "Success",
            Self::Failed => "Failed",
        }
    }

    pub fn style(&self) -> Style {
        match self {
            Self::Processing => Style::default().fg(Color::Yellow),
            Self::Success => Style::default().fg(Color::Green),
            Self::Failed => Style::default().fg(Color::Red),
        }
    }
}

/// Domain model for a known client.
#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ClientStatus,
    pub requests_sent: u64,
    pub last_activity: DateTime<Utc>,
}

impl Client {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        status: ClientStatus,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            status,
            requests_sent: 0,
            last_activity: Utc::now(),
        }
    }
}

/// Domain model for a known MCP server.
#[derive(Debug, Clone)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ServerStatus,
    pub requests_received: u64,
    pub last_activity: DateTime<Utc>,
}

impl Server {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        status: ServerStatus,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            status,
            requests_received: 0,
            last_activity: Utc::now(),
        }
    }
}

/// Item rendered in the activity feed.
#[derive(Debug, Clone)]
pub struct ActivityItem {
    pub timestamp: DateTime<Utc>,
    pub client: String,
    pub server: String,
    pub action: String,
    pub status: ActivityStatus,
}
