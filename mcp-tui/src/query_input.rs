use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct QueryInput {
    placeholder: String,
}

impl QueryInput {
    pub fn new() -> Self {
        Self {
            placeholder: "Enter a query to send to MCP…".to_string(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, value: &str, focused: bool) {
        let mut block = Block::default().title("Query").borders(Borders::ALL);
        if focused {
            block = block.border_style(Style::default().fg(Color::Cyan));
        }

        let display = if value.is_empty() {
            Line::from(Span::styled(
                &self.placeholder,
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            Line::from(Span::raw(value.to_string()))
        };

        let paragraph = Paragraph::new(display).block(block);
        frame.render_widget(paragraph, area);
    }
}
