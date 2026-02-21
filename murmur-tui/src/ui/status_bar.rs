//! Status bar at the bottom of the screen.

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        format!(
            " {} posts │ {} peers │ /help for commands",
            app.timeline.len(),
            app.peer_count,
        )
    };

    let status_bar = Paragraph::new(format!(" {}", status))
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(status_bar, area);
}
