use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::components::ActivityItem;

pub struct ActivityFeed {
    state: ListState,
}

impl ActivityFeed {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { state }
    }

    pub fn focus(&mut self) {
        if self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self, len: usize) {
        let idx = self.state.selected().unwrap_or(0);
        if len == 0 {
            self.state.select(None);
            return;
        }
        let next = if idx + 1 >= len { len - 1 } else { idx + 1 };
        self.state.select(Some(next));
    }

    pub fn previous(&mut self) {
        let idx = self.state.selected().unwrap_or(0);
        self.state.select(Some(idx.saturating_sub(1)));
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        activities: &[ActivityItem],
        focused: bool,
    ) {
        let block = Block::default()
            .title("Activity Feed")
            .borders(Borders::ALL)
            .border_style(border_style(focused));

        let items: Vec<ListItem> = activities
            .iter()
            .rev()
            .map(|activity| list_item(activity))
            .collect();

        let mut state = self.state.clone();
        // Ensure selection stays inside bounds after updates.
        if let Some(idx) = state.selected() {
            let max_index = items.len().saturating_sub(1);
            state.select(Some(idx.min(max_index)));
        }

        frame.render_stateful_widget(List::new(items).block(block), area, &mut state);
        self.state = state;
    }
}

fn list_item(item: &ActivityItem) -> ListItem<'static> {
    let status_style = item.status.style();
    let timestamp = item.timestamp.format("%H:%M:%S");
    let content = vec![Line::from(vec![
        Span::styled(
            format!("[{}] ", timestamp),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("{} → {} ", item.client, item.server),
            Style::default(),
        ),
        Span::styled(item.action.clone(), Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(format!("[{}]", item.status.label()), status_style),
    ])];
    ListItem::new(content)
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}
