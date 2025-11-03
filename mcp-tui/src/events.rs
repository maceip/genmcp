use anyhow::{Context, Result};
use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use tokio::task;
use tracing::warn;

/// High level events understood by the application.
#[derive(Debug, Clone)]
pub enum Event {
    Quit,
    /// Textual input from the user.
    Input(char),
    Enter,
    Backspace,
    Tab,
    FocusNext,
    FocusPrev,
    Up,
    Down,
    Left,
    Right,
}

/// Blocking event reader wrapped for async callers.
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn next(&mut self) -> Result<Event> {
        loop {
            let event = match task::spawn_blocking(event::read)
                .await
                .context("failed to join tui event reader task")?
            {
                Ok(event) => event,
                Err(err) => {
                    warn!("tui event reader unavailable: {err}");
                    return Ok(Event::Quit);
                }
            };

            if let Some(app_event) = map_event(event) {
                return Ok(app_event);
            }
        }
    }
}

fn map_event(event: CrosstermEvent) -> Option<Event> {
    match event {
        CrosstermEvent::Key(KeyEvent {
            code,
            modifiers,
            kind,
            ..
        }) => {
            if kind != KeyEventKind::Press {
                return None;
            }
            match code {
                KeyCode::Esc => Some(Event::Quit),
                KeyCode::Enter => Some(Event::Enter),
                KeyCode::Tab => {
                    if modifiers.contains(KeyModifiers::SHIFT) {
                        Some(Event::FocusPrev)
                    } else if modifiers.contains(KeyModifiers::CONTROL) {
                        Some(Event::FocusNext)
                    } else {
                        Some(Event::Tab)
                    }
                }
                KeyCode::BackTab => Some(Event::FocusPrev),
                KeyCode::Backspace => Some(Event::Backspace),
                KeyCode::Left => Some(Event::Left),
                KeyCode::Right => Some(Event::Right),
                KeyCode::Up => Some(Event::Up),
                KeyCode::Down => Some(Event::Down),
                KeyCode::Char('c') | KeyCode::Char('q')
                    if modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    Some(Event::Quit)
                }
                KeyCode::Char(character) => Some(Event::Input(character)),
                _ => None,
            }
        }
        _ => None,
    }
}
