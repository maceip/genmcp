use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

#[derive(Clone, Debug)]
pub struct QuickAction {
    pub label: String,
    pub description: String,
    pub command: String,
}

pub struct QuickAccess {
    items: Vec<QuickAction>,
    state: ListState,
}

impl QuickAccess {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            items: default_actions(),
            state,
        }
    }

    pub fn focus(&mut self) {
        if self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let idx = self.state.selected().unwrap_or(0);
        let next = if idx + 1 >= self.items.len() {
            0
        } else {
            idx + 1
        };
        self.state.select(Some(next));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let idx = self.state.selected().unwrap_or(0);
        let prev = if idx == 0 {
            self.items.len() - 1
        } else {
            idx - 1
        };
        self.state.select(Some(prev));
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool) {
        let mut block = Block::default()
            .title("Quick Actions")
            .borders(Borders::ALL);
        if focused {
            block = block.border_style(Style::default().fg(Color::Cyan));
        }
        let items: Vec<ListItem> = self.items.iter().map(render_item).collect();

        let mut state = self.state.clone();
        if let Some(idx) = state.selected() {
            let max = items.len().saturating_sub(1);
            state.select(Some(idx.min(max)));
        }

        frame.render_stateful_widget(List::new(items).block(block), area, &mut state);
        self.state = state;
    }

    pub fn execute_selected_action(&mut self) -> Option<String> {
        self.state
            .selected()
            .and_then(|idx| self.items.get(idx))
            .map(|item| item.command.clone())
    }
}

fn render_item(action: &QuickAction) -> ListItem<'static> {
    let lines = vec![
        Line::from(Span::styled(
            action.label.clone(),
            Style::default().fg(Color::White),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                action.description.clone(),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    ListItem::new(lines)
}

fn default_actions() -> Vec<QuickAction> {
    vec![
        QuickAction {
            label: "List available tools".to_string(),
            description: "Inspect which tools MCP exposes".to_string(),
            command: "list_tools".to_string(),
        },
        QuickAction {
            label: "Check server health".to_string(),
            description: "Gather latest health metrics".to_string(),
            command: "check_health".to_string(),
        },
        QuickAction {
            label: "Open session".to_string(),
            description: "Start a new interactive session".to_string(),
            command: "open_session".to_string(),
        },
    ]
}
