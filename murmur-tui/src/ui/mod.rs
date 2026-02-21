pub mod timeline;
pub mod compose;
pub mod thread;
pub mod profile;
pub mod status_bar;

use crate::app::{App, InputMode, Screen};
use ratatui::Frame;

/// Render the full UI.
pub fn draw(frame: &mut Frame, app: &App) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(3), // Input / compose
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    // Header
    draw_header(frame, app, chunks[0]);

    // Main content area
    match &app.screen {
        Screen::Timeline => timeline::draw(frame, app, chunks[1]),
        Screen::Thread(_) => thread::draw(frame, app, chunks[1]),
        Screen::Compose => {
            timeline::draw(frame, app, chunks[1]);
        }
        Screen::Profile => profile::draw(frame, app, chunks[1]),
    }

    // Input area
    compose::draw(frame, app, chunks[2]);

    // Status bar
    status_bar::draw(frame, app, chunks[3]);
}

fn draw_header(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let mode_indicator = match app.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Editing => "COMPOSE",
    };

    let screen_name = match &app.screen {
        Screen::Timeline => "Timeline",
        Screen::Compose => "Compose",
        Screen::Thread(_) => "Thread",
        Screen::Profile => "Profile",
    };

    let header_text = format!(
        " murmur │ {} │ {} │ {} peers │ {}",
        screen_name,
        mode_indicator,
        app.peer_count,
        app.identity.pubkey_short()
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(header, area);
}
