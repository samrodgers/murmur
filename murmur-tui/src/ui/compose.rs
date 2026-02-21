//! Post composition input area.

use crate::app::{App, InputMode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let (title, style) = match app.input_mode {
        InputMode::Normal => (
            " Press 'n' to compose, ':' for commands ",
            Style::default().fg(Color::DarkGray),
        ),
        InputMode::Editing => {
            let chars_left = 500usize.saturating_sub(app.input.chars().count());
            (
                " Composing (Esc to cancel, Enter to send) ",
                if chars_left == 0 {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Yellow)
                },
            )
        }
    };

    let display_text = if app.input_mode == InputMode::Editing {
        let chars_left = 500usize.saturating_sub(app.input.chars().count());
        format!("{} [{}]", app.input, chars_left)
    } else if app.input.is_empty() {
        String::new()
    } else {
        app.input.clone()
    };

    let input = Paragraph::new(display_text).style(style).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(
                if app.input_mode == InputMode::Editing {
                    Color::Yellow
                } else {
                    Color::DarkGray
                },
            )),
    );

    frame.render_widget(input, area);

    // Show cursor when editing
    if app.input_mode == InputMode::Editing {
        frame.set_cursor_position((
            area.x + app.cursor_pos as u16 + 1,
            area.y + 1,
        ));
    }
}
