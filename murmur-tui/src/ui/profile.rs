//! Profile display.
//! Phase 1: Minimal — just shows pubkey and post count.

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let own_count = app
        .db
        .own_event_count()
        .unwrap_or(0);
    let total_count = app
        .db
        .total_event_count()
        .unwrap_or(0);

    let text = format!(
        "\n  Public Key:  {}\n\n  \
         Peer ID:     {}\n\n  \
         Your Posts:  {}\n  \
         Total Events: {}\n  \
         Peers:       {}\n\n  \
         Data Dir:    {}",
        app.identity.pubkey_hex(),
        app.node.peer_id(),
        own_count,
        total_count,
        app.peer_count,
        app.data_dir.display(),
    );

    let profile = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Profile ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(profile, area);
}
