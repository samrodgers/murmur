//! Murmur TUI — Terminal interface for the Murmur decentralized social protocol.
//!
//! Usage:
//!   MURMUR_DATA=./node-alice cargo run -p murmur-tui
//!   MURMUR_DATA=./node-bob cargo run -p murmur-tui -- --port 9001

mod app;
mod commands;
mod ui;

use app::{App, InputMode, Screen};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    // Parse data directory from env or args
    let data_dir = std::env::var("MURMUR_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./murmur-data"));

    // Parse port from args (default 9000)
    let port = parse_port();

    // Set up terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Create application
    let mut app = App::new(data_dir, port).await?;

    // Main event loop
    let result = run_loop(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    // Clean shutdown
    let _ = app.node.shutdown().await;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        // Process any pending network events
        app.process_network_events().await;

        // Draw the UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Poll for input events with a short timeout so we can
        // process network events frequently
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                // Only handle key press events (not release/repeat)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Clear status message on any keypress
                app.status_message = None;

                match app.input_mode {
                    InputMode::Normal => {
                        handle_normal_mode(app, key.code, key.modifiers).await;
                    }
                    InputMode::Editing => {
                        handle_editing_mode(app, key.code, key.modifiers).await;
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

async fn handle_normal_mode(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        // Quit
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.select_down(),
        KeyCode::Char('k') | KeyCode::Up => app.select_up(),

        // New post
        KeyCode::Char('n') => {
            app.screen = Screen::Compose;
            app.input_mode = InputMode::Editing;
            app.input.clear();
            app.cursor_pos = 0;
        }

        // Reply to selected
        KeyCode::Char('r') => {
            if app.selected_event().is_some() {
                app.input = "/reply ".to_string();
                app.cursor_pos = app.input.len();
                app.input_mode = InputMode::Editing;
            } else {
                app.status_message = Some("No post selected to reply to".into());
            }
        }

        // View thread
        KeyCode::Enter => {
            if let Some(event) = app.selected_event() {
                let id = event.id;
                app.load_thread(&id);
            }
        }

        // Back to timeline
        KeyCode::Esc => {
            if app.screen != Screen::Timeline {
                app.screen = Screen::Timeline;
                app.current_thread = None;
                app.selected = 0;
                app.refresh_timeline();
            }
        }

        // Command mode
        KeyCode::Char(':') | KeyCode::Char('/') => {
            app.input = "/".to_string();
            app.cursor_pos = 1;
            app.input_mode = InputMode::Editing;
        }

        // Profile
        KeyCode::Char('i') => {
            app.screen = Screen::Profile;
        }

        // Refresh
        KeyCode::Char('R') => {
            app.refresh_timeline();
            app.status_message = Some("Timeline refreshed".into());
        }

        _ => {}
    }
}

async fn handle_editing_mode(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) {
    match key {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input.clear();
            app.cursor_pos = 0;
            if app.screen == Screen::Compose {
                app.screen = Screen::Timeline;
            }
        }

        KeyCode::Enter => {
            let input = app.input.clone();
            app.input.clear();
            app.cursor_pos = 0;
            app.input_mode = InputMode::Normal;

            if input.starts_with('/') {
                // Command mode
                if let Some(msg) = commands::execute_command(app, &input).await {
                    app.status_message = Some(msg);
                }
            } else if !input.trim().is_empty() {
                // Direct post
                match app.publish_post(&input).await {
                    Ok(()) => {
                        app.status_message = Some("Post published!".into());
                    }
                    Err(e) => {
                        app.status_message = Some(format!("Failed to post: {e}"));
                    }
                }
            }

            if app.screen == Screen::Compose {
                app.screen = Screen::Timeline;
            }
        }

        KeyCode::Char(c) => {
            if app.input.chars().count() < 500 || app.input.starts_with('/') {
                app.input.insert(app.cursor_pos, c);
                app.cursor_pos += 1;
            }
        }

        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
                app.input.remove(app.cursor_pos);
            }
        }

        KeyCode::Left => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
        }

        KeyCode::Right => {
            if app.cursor_pos < app.input.len() {
                app.cursor_pos += 1;
            }
        }

        KeyCode::Home => {
            app.cursor_pos = 0;
        }

        KeyCode::End => {
            app.cursor_pos = app.input.len();
        }

        _ => {}
    }
}

fn parse_port() -> u16 {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--port" || args[i] == "-p" {
            if let Some(port_str) = args.get(i + 1) {
                if let Ok(port) = port_str.parse() {
                    return port;
                }
            }
        }
    }

    // Check MURMUR_PORT env var
    if let Ok(port_str) = std::env::var("MURMUR_PORT") {
        if let Ok(port) = port_str.parse() {
            return port;
        }
    }

    9000 // default
}
