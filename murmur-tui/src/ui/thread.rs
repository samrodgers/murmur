//! Thread view — a post with its replies.

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let Some((root, replies)) = &app.current_thread else {
        let empty = Paragraph::new("No thread loaded")
            .block(Block::default().title(" Thread ").borders(Borders::ALL));
        frame.render_widget(empty, area);
        return;
    };

    let own_pubkey_hex = app.identity.pubkey_hex();

    let mut items: Vec<ListItem> = Vec::new();

    // Root post (always first, highlighted)
    let is_own = root.pubkey.to_hex() == own_pubkey_hex;
    let root_item = {
        let author = root.pubkey.short();
        let time = format_relative_time(root.created_at);
        let marker = if app.selected == 0 { "▸ " } else { "  " };

        let style = if app.selected == 0 {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(marker.to_string(), style),
                Span::styled(
                    format!("● {} ", if is_own { format!("{} (you)", author) } else { author }),
                    Style::default()
                        .fg(if is_own { Color::Cyan } else { Color::Green })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("· {}", time), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(root.content.clone(), style),
            ]),
            Line::from(""),
        ])
    };
    items.push(root_item);

    // Replies
    for (i, reply) in replies.iter().enumerate() {
        let idx = i + 1;
        let is_own_reply = reply.pubkey.to_hex() == own_pubkey_hex;
        let author = reply.pubkey.short();
        let time = format_relative_time(reply.created_at);
        let marker = if app.selected == idx { "▸ " } else { "  " };

        let style = if app.selected == idx {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let item = ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("  {}↳ ", marker), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} ",
                        if is_own_reply {
                            format!("{} (you)", author)
                        } else {
                            author
                        }
                    ),
                    Style::default()
                        .fg(if is_own_reply { Color::Cyan } else { Color::Green })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("· {}", time), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("      "),
                Span::styled(reply.content.clone(), style),
            ]),
        ]);
        items.push(item);
    }

    let title = format!(" Thread ({} replies) ", replies.len());
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
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
