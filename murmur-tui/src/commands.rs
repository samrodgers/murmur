//! CLI-style commands for the TUI.
//! Commands are entered with a / prefix in the compose input.

use crate::app::App;

/// Parse and execute a command string.
/// Returns Some(message) if a command was recognized, None otherwise.
pub async fn execute_command(app: &mut App, input: &str) -> Option<String> {
    let input = input.trim();
    if !input.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).map(|s| s.trim()).unwrap_or("");

    match cmd {
        "/post" | "/p" => {
            if args.is_empty() {
                Some("Usage: /post <message>".into())
            } else if args.chars().count() > 500 {
                Some(format!(
                    "Post too long: {} chars (max 500)",
                    args.chars().count()
                ))
            } else {
                match app.publish_post(args).await {
                    Ok(()) => Some("Post published!".into()),
                    Err(e) => Some(format!("Failed to post: {e}")),
                }
            }
        }

        "/reply" | "/r" => {
            if let Some(event) = app.selected_event() {
                let parent_id = event.id;
                if args.is_empty() {
                    Some("Usage: /reply <message> (replies to selected post)".into())
                } else {
                    match app.publish_reply(args, parent_id).await {
                        Ok(()) => Some("Reply published!".into()),
                        Err(e) => Some(format!("Failed to reply: {e}")),
                    }
                }
            } else {
                Some("No post selected to reply to".into())
            }
        }

        "/peers" => match app.node.get_peers().await {
            Ok(peers) => {
                if peers.is_empty() {
                    Some("No connected peers".into())
                } else {
                    let list: Vec<String> = peers.iter().map(|p| format!("  {p}")).collect();
                    Some(format!("Connected peers ({}):\n{}", peers.len(), list.join("\n")))
                }
            }
            Err(e) => Some(format!("Failed to get peers: {e}")),
        },

        "/whoami" | "/id" => Some(format!(
            "Public key: {}\nPeer ID: {}\nData dir: {}",
            app.identity.pubkey_hex(),
            app.node.peer_id(),
            app.data_dir.display()
        )),

        "/thread" | "/t" => {
            if let Some(event) = app.selected_event() {
                let id = event.id;
                app.load_thread(&id);
                Some("Viewing thread".into())
            } else {
                Some("No post selected".into())
            }
        }

        "/stats" => {
            let total = app
                .db
                .total_event_count()
                .unwrap_or(-1);
            let cached = app
                .db
                .cached_event_count()
                .unwrap_or(-1);
            Some(format!(
                "Events: {} total ({} cached from others)\nPeers: {}",
                total, cached, app.peer_count
            ))
        }

        "/help" | "/h" | "/?" => Some(
            "Commands:\n  \
             /post <msg>  - Publish a post (or press 'n')\n  \
             /reply <msg> - Reply to selected post (or press 'r')\n  \
             /thread      - View thread for selected post (or press Enter)\n  \
             /peers       - List connected peers\n  \
             /whoami      - Show your identity\n  \
             /stats       - Show database statistics\n  \
             /help        - Show this help\n\n\
             Keys:\n  \
             j/k or ↑/↓   - Navigate timeline\n  \
             n             - New post\n  \
             r             - Reply to selected\n  \
             Enter         - View thread\n  \
             Esc           - Back / cancel\n  \
             q             - Quit"
                .into(),
        ),

        _ => Some(format!("Unknown command: {cmd}. Type /help for available commands.")),
    }
}
