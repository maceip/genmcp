use std::collections::HashMap;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::{
    components::{
        ActivityFeed, ActivityItem, Client, ClientsPanel, FocusArea, QueryInput, QuickAccess,
        Server, ServersPanel,
    },
    events::Event,
};

const FOCUS_ORDER: [FocusArea; 5] = [
    FocusArea::Clients,
    FocusArea::Servers,
    FocusArea::Activity,
    FocusArea::QuickAccess,
    FocusArea::QueryInput,
];

pub struct NavigationContext {
    pub client_len: usize,
    pub server_len: usize,
    pub activity_len: usize,
}

pub struct UI {
    focus: FocusArea,
    pub clients_panel: ClientsPanel,
    pub servers_panel: ServersPanel,
    pub activity_feed: ActivityFeed,
    pub query_input: QueryInput,
    pub quick_access: QuickAccess,
}

impl UI {
    pub fn new() -> Self {
        Self {
            focus: FocusArea::Clients,
            clients_panel: ClientsPanel::new(),
            servers_panel: ServersPanel::new(),
            activity_feed: ActivityFeed::new(),
            query_input: QueryInput::new(),
            quick_access: QuickAccess::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        frame: &mut Frame,
        clients: &HashMap<String, Client>,
        servers: &HashMap<String, Server>,
        activities: &[ActivityItem],
        query_input: &str,
    ) {
        let area = frame.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(5)].as_ref())
            .split(area);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
            .split(chunks[0]);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunks[0]);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)].as_ref())
            .split(main_chunks[1]);

        self.clients_panel
            .render(frame, left[0], clients, self.focus == FocusArea::Clients);
        self.servers_panel
            .render(frame, left[1], servers, self.focus == FocusArea::Servers);
        self.activity_feed.render(
            frame,
            right[0],
            activities,
            self.focus == FocusArea::Activity,
        );
        self.quick_access
            .render(frame, right[1], self.focus == FocusArea::QuickAccess);
        self.query_input.render(
            frame,
            chunks[1],
            query_input,
            self.focus == FocusArea::QueryInput,
        );
    }

    pub fn get_focus(&self) -> FocusArea {
        self.focus
    }

    pub fn cycle_focus(&mut self) {
        self.focus_next();
    }

    pub fn focus_next(&mut self) {
        let idx = focus_index(self.focus);
        let next = FOCUS_ORDER[(idx + 1) % FOCUS_ORDER.len()];
        self.set_focus(next);
    }

    pub fn focus_prev(&mut self) {
        let idx = focus_index(self.focus);
        let prev = if idx == 0 {
            FOCUS_ORDER[FOCUS_ORDER.len() - 1]
        } else {
            FOCUS_ORDER[idx - 1]
        };
        self.set_focus(prev);
    }

    pub fn handle_navigation(&mut self, ctx: NavigationContext, event: Event) -> bool {
        match event {
            Event::Up => {
                match self.focus {
                    FocusArea::Clients => self.clients_panel.previous(ctx.client_len),
                    FocusArea::Servers => self.servers_panel.previous(ctx.server_len),
                    FocusArea::Activity => self.activity_feed.previous(),
                    FocusArea::QuickAccess => self.quick_access.previous(),
                    FocusArea::QueryInput => return false,
                }
                true
            }
            Event::Down => {
                match self.focus {
                    FocusArea::Clients => self.clients_panel.next(ctx.client_len),
                    FocusArea::Servers => self.servers_panel.next(ctx.server_len),
                    FocusArea::Activity => self.activity_feed.next(ctx.activity_len),
                    FocusArea::QuickAccess => self.quick_access.next(),
                    FocusArea::QueryInput => return false,
                }
                true
            }
            Event::Left => {
                let next_focus = match self.focus {
                    FocusArea::Servers => Some(FocusArea::Clients),
                    FocusArea::Activity => Some(FocusArea::Clients),
                    FocusArea::QuickAccess => Some(FocusArea::Activity),
                    FocusArea::QueryInput => Some(FocusArea::QuickAccess),
                    FocusArea::Clients => None,
                };
                if let Some(focus) = next_focus {
                    self.set_focus(focus);
                    return true;
                }
                false
            }
            Event::Right => {
                let next_focus = match self.focus {
                    FocusArea::Clients => Some(FocusArea::Servers),
                    FocusArea::Servers => Some(FocusArea::Activity),
                    FocusArea::Activity => Some(FocusArea::QuickAccess),
                    FocusArea::QuickAccess => Some(FocusArea::QueryInput),
                    FocusArea::QueryInput => None,
                };
                if let Some(focus) = next_focus {
                    self.set_focus(focus);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn set_focus(&mut self, focus: FocusArea) {
        self.focus = focus;
        match self.focus {
            FocusArea::QuickAccess => self.quick_access.focus(),
            FocusArea::Activity => self.activity_feed.focus(),
            _ => {}
        }
    }
}

fn focus_index(focus: FocusArea) -> usize {
    FOCUS_ORDER
        .iter()
        .position(|item| *item == focus)
        .unwrap_or(0)
}
