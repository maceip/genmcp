use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::components::Server;

pub struct ServersPanel {
    state: ListState,
}

impl ServersPanel {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { state }
    }

    pub fn next(&mut self, len: usize) {
        if len == 0 {
            self.state.select(None);
            return;
        }
        let idx = self.state.selected().unwrap_or(0);
        let next = if idx + 1 >= len { 0 } else { idx + 1 };
        self.state.select(Some(next));
    }

    pub fn previous(&mut self, len: usize) {
        if len == 0 {
            self.state.select(None);
            return;
        }
        let idx = self.state.selected().unwrap_or(0);
        let prev = if idx == 0 { len - 1 } else { idx - 1 };
        self.state.select(Some(prev));
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        servers: &HashMap<String, Server>,
        focused: bool,
    ) {
        let mut items: Vec<&Server> = servers.values().collect();
        items.sort_by(|a, b| a.name.cmp(&b.name));

        let rendered: Vec<ListItem> = items.iter().map(|server| render_item(server)).collect();
        let mut block = Block::default().title("Servers").borders(Borders::ALL);
        if focused {
            block = block.border_style(Style::default().fg(Color::Cyan));
        }
        let mut state = self.state.clone();
        if let Some(selected) = state.selected() {
            let max = rendered.len().saturating_sub(1);
            state.select(Some(selected.min(max)));
        }
        frame.render_stateful_widget(List::new(rendered).block(block), area, &mut state);
        self.state = state;
    }
}

fn render_item(server: &Server) -> ListItem<'static> {
    let content = vec![
        Line::from(vec![
            Span::styled(server.name.clone(), Style::default().fg(Color::White)),
            Span::raw(" "),
            Span::styled(
                format!("[{}]", server.status.label()),
                server.status.style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                server.description.clone(),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    ListItem::new(content)
}
