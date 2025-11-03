use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

// MCP Gateway integration
use mcp_common::types::{ProxySession, SessionId, LogEntry};
use mcp_core::{McpClient, ServerInfo, ClientConfig, TransportConfig};

use crate::components::{ActivityItem, Client, Server};
use crate::events::{Event, EventHandler};
use crate::ui::{NavigationContext, UI};

/// Main application state
pub struct App {
    /// UI state management
    pub ui: UI,
    /// Event handler for user input
    pub events: EventHandler,
    /// Connected clients (AI assistants, tools)
    pub clients: HashMap<String, Client>,
    /// Available servers (backend services)
    pub servers: HashMap<String, Server>,
    /// Activity feed items
    pub activities: Vec<ActivityItem>,
    /// Current query input
    pub query_input: String,
    /// Application running state
    pub running: bool,
    /// Last update time
    pub last_update: Instant,
    
    // MCP Gateway integration
    /// MCP client for gateway communication
    pub gateway_client: Option<Arc<McpClient>>,
    /// Active proxy sessions
    pub proxy_sessions: HashMap<SessionId, ProxySession>,
    /// Real-time activity log
    pub activity_log: Vec<LogEntry>,
    /// Connected MCP servers info
    pub mcp_servers: HashMap<String, ServerInfo>,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> Result<Self> {
        debug!("Initializing MCP TUI application");

        let ui = UI::new();
        let events = EventHandler::new();

        Ok(Self {
            ui,
            events,
            clients: HashMap::new(),
            servers: HashMap::new(),
            activities: Vec::new(),
            query_input: String::new(),
            running: true,
            last_update: Instant::now(),
            
            // MCP Gateway integration
            gateway_client: None,
            proxy_sessions: HashMap::new(),
            activity_log: Vec::new(),
            mcp_servers: HashMap::new(),
        })
    }

    /// Run the main application loop
    pub async fn run(&mut self) -> Result<()> {
        debug!("Starting application run loop");

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initialize with sample data for demonstration
        self.init_sample_data();

        // Main event loop
        while self.running {
            // Draw the UI
            terminal.draw(|f| {
                self.ui.draw(
                    f,
                    &self.clients,
                    &self.servers,
                    &self.activities,
                    &self.query_input,
                );
            })?;

            // Handle events with timeout
            match crossterm::event::poll(Duration::from_millis(100)) {
                Ok(true) => {
                    if let Ok(event) = self.events.next().await {
                        self.handle_event(event).await?;
                    }
                }
                Ok(false) => {}
                Err(err) => {
                    warn!("input polling failed: {err}");
                    self.running = false;
                }
            }

            // Update state periodically
            if self.last_update.elapsed() > Duration::from_secs(1) {
                self.update_state().await;
                self.last_update = Instant::now();
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
    
    /// Initialize MCP gateway connection
    async fn init_gateway(&mut self) -> Result<()> {
        info!("Initializing MCP gateway connection");
        
        // Create transport configuration for HTTP
        let transport_config = TransportConfig::HttpSse(
            mcp_core::transport::HttpSseConfig {
                url: "http://localhost:8080".to_string(),
                headers: std::collections::HashMap::new(),
            }
        );
        
        // Create client configuration
        let client_config = ClientConfig::default();
        
        // Create notification handler
        let notification_handler = Box::new(mcp_core::notification::DefaultNotificationHandler);
        
        // Initialize MCP client
        match McpClient::new(transport_config, client_config, notification_handler).await {
            Ok(client) => {
                self.gateway_client = Some(Arc::new(client));
                info!("Successfully connected to MCP gateway");
                
                // Load real clients, servers, and activities from gateway
                self.load_gateway_data().await?;
            }
            Err(e) => {
                warn!("Failed to connect to MCP gateway: {}", e);
                // Continue without gateway connection for now
            }
        }
        
        Ok(())
    }

    /// Handle user input events
    async fn handle_event(&mut self, event: Event) -> Result<()> {
        debug!("Handling event: {:?}", event);

        // Let UI handle navigation first
        let nav_ctx = NavigationContext {
            client_len: self.clients.len(),
            server_len: self.servers.len(),
            activity_len: self.activities.len(),
        };

        if self.ui.handle_navigation(nav_ctx, event.clone()) {
            return Ok(());
        }

        // Handle non-navigation events
        match event {
            Event::Quit => {
                self.running = false;
            }
            Event::Input(character) => {
                if self.ui.get_focus() == crate::components::FocusArea::QueryInput {
                    self.query_input.push(character);
                }
            }
            Event::Backspace => {
                if self.ui.get_focus() == crate::components::FocusArea::QueryInput {
                    self.query_input.pop();
                }
            }
            Event::Enter => {
                match self.ui.get_focus() {
                    crate::components::FocusArea::QueryInput => {
                        if !self.query_input.is_empty() {
                            self.process_query().await;
                            self.query_input.clear();
                        }
                    }
                    crate::components::FocusArea::QuickAccess => {
                        if let Some(message) = self.ui.quick_access.execute_selected_action() {
                            // Add the action result to activity feed
                            let activity = crate::components::ActivityItem {
                                timestamp: chrono::Utc::now(),
                                client: "User".to_string(),
                                server: "System".to_string(),
                                action: message,
                                status: crate::components::ActivityStatus::Success,
                            };
                            self.activities.push(activity);
                        }
                    }
                    _ => {}
                }
            }
            Event::Tab => {
                self.ui.cycle_focus();
            }
            Event::FocusNext => {
                self.ui.focus_next();
            }
            Event::FocusPrev => {
                self.ui.focus_prev();
            }
            _ => {}
        }

        Ok(())
    }

    /// Process user query from input
    async fn process_query(&mut self) {
        let query = self.query_input.clone();
        info!("Processing query: {}", query);

        // Add to activity feed
        let activity = ActivityItem {
            timestamp: chrono::Utc::now(),
            client: "User".to_string(),
            server: "System".to_string(),
            action: format!("Query: {}", query),
            status: crate::components::ActivityStatus::Processing,
        };

        self.activities.push(activity);

        // Process query (placeholder for actual LLM integration)
        // TODO: Integrate with mcp-llm for natural language processing
    }

    /// Update application state
    async fn update_state(&mut self) {
        // Sync with MCP gateway state
        if let Some(client) = &self.gateway_client {
            // Check for new activity log entries
            match client.get_activity_log().await {
                Ok(new_entries) => {
                    // Only add entries that are newer than our latest activity
                    if let Some(latest_activity) = self.activities.last() {
                        for entry in new_entries {
                            if entry.timestamp > latest_activity.timestamp {
                                let activity = self.log_entry_to_activity(&entry);
                                self.activities.push(activity);
                            }
                        }
                    } else {
                        // No existing activities, add all recent entries
                        for entry in new_entries.into_iter().take(20) {
                            let activity = self.log_entry_to_activity(&entry);
                            self.activities.push(activity);
                        }
                    }
                }
                Err(e) => warn!("Failed to sync activity log: {}", e),
            }
            
            // Update client/server statuses
            match client.list_sessions().await {
                Ok(sessions) => {
                    for session in sessions {
                        if let Some(client) = self.clients.get_mut(&session.id) {
                            client.status = if session.status == SessionStatus::Active {
                                crate::components::ClientStatus::Connected
                            } else {
                                crate::components::ClientStatus::Disconnected
                            };
                            client.requests_sent = session.request_count;
                            client.last_activity = session.last_activity;
                        }
                    }
                }
                Err(e) => warn!("Failed to update session status: {}", e),
            }
            
            match client.list_servers().await {
                Ok(servers) => {
                    for server in servers {
                        if let Some(ui_server) = self.servers.get_mut(&server.id) {
                            ui_server.status = if server.is_healthy {
                                crate::components::ServerStatus::Running
                            } else {
                                crate::components::ServerStatus::Error
                            };
                            ui_server.requests_received = server.request_count;
                            ui_server.last_activity = server.last_activity;
                        }
                    }
                }
                Err(e) => warn!("Failed to update server status: {}", e),
            }
        }
        
        // Clean up old activities to prevent memory issues
        if self.activities.len() > 100 {
            self.activities.drain(0..50);
        }
    }

    /// Initialize with sample data for demonstration
    fn init_sample_data(&mut self) {
        // Sample clients
        self.clients.insert(
            "ai-assistant".to_string(),
            Client {
                id: "ai-assistant".to_string(),
                name: "AI Assistant".to_string(),
                description: "General purpose AI assistant".to_string(),
                status: crate::components::ClientStatus::Connected,
                requests_sent: 12,
                last_activity: chrono::Utc::now(),
            },
        );

        self.clients.insert(
            "code-editor".to_string(),
            Client {
                id: "code-editor".to_string(),
                name: "Code Editor".to_string(),
                description: "Development environment tools".to_string(),
                status: crate::components::ClientStatus::Connected,
                requests_sent: 8,
                last_activity: chrono::Utc::now(),
            },
        );

        // Sample servers
        self.servers.insert(
            "python-server".to_string(),
            Server {
                id: "python-server".to_string(),
                name: "Python Server".to_string(),
                description: "Python execution environment".to_string(),
                status: crate::components::ServerStatus::Running,
                requests_received: 15,
                last_activity: chrono::Utc::now(),
            },
        );

        self.servers.insert(
            "database".to_string(),
            Server {
                id: "database".to_string(),
                name: "Database".to_string(),
                description: "PostgreSQL database".to_string(),
                status: crate::components::ServerStatus::Running,
                requests_received: 22,
                last_activity: chrono::Utc::now(),
            },
        );

        // Sample activities
        let now = chrono::Utc::now();
        self.activities.push(ActivityItem {
            timestamp: now - chrono::Duration::minutes(2),
            client: "AI Assistant".to_string(),
            server: "Python Server".to_string(),
            action: "get_weather()".to_string(),
            status: crate::components::ActivityStatus::Success,
        });

        self.activities.push(ActivityItem {
            timestamp: now - chrono::Duration::minutes(5),
            client: "Code Editor".to_string(),
            server: "Database".to_string(),
            action: "SELECT * FROM users".to_string(),
            status: crate::components::ActivityStatus::Processing,
        });
    }
}
