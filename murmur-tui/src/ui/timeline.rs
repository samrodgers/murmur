//! Post list view — the main timeline.

use crate::app::App;
use murmur_core::event::types::{Event, EventKind};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .timeline
        .iter()
        .enumerate()
        .map(|(i, event)| {
            let is_selected = i == app.selected;
            event_to_list_item(event, is_selected, &app.identity.pubkey_hex())
        })
        .collect();

    let title = if app.timeline.is_empty() {
        " Timeline (empty — waiting for posts) "
    } else {
        " Timeline "
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

fn event_to_list_item<'a>(event: &Event, selected: bool, own_pubkey_hex: &str) -> ListItem<'a> {
    let author = event.pubkey.short();
    let is_own = event.pubkey.to_hex() == own_pubkey_hex;
    let author_display = if is_own {
        format!("{} (you)", author)
    } else {
        author
    };

    let kind_marker = match event.kind {
        EventKind::Post => "●",
        EventKind::Reply => "↳",
        EventKind::Repost => "↻",
        EventKind::Reaction => "♥",
        _ => "•",
    };

    let time = format_relative_time(event.created_at);

    let marker = if selected { "▸ " } else { "  " };

    let style = if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if is_own {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    let line = Line::from(vec![
        Span::styled(marker.to_string(), style),
        Span::styled(
            format!("{} ", kind_marker),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("{} ", author_display),
            Style::default()
                .fg(if is_own { Color::Cyan } else { Color::Green })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("· {} ", time),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(event.content.clone(), style),
    ]);

    ListItem::new(line)
}

fn format_relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 0 {
        "just now".into()
    } else if diff < 60 {
        format!("{}s", diff)
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else if diff < 86400 {
        format!("{}h", diff / 3600)
    } else {
        format!("{}d", diff / 86400)
    }
}
