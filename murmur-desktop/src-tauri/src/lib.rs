mod commands;
mod state;
mod types;

use state::AppState;
use std::path::PathBuf;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    // Create a multi-threaded tokio runtime for async operations.
    // The Handle is stored as managed state for commands; the Runtime
    // itself is moved to the network listener thread to keep it alive.
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let rt_handle = rt.handle().clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./murmur-data"));
            std::fs::create_dir_all(&data_dir)?;

            let mut app_state = rt_handle.block_on(async { AppState::new(data_dir).await })?;

            // If an identity already exists, start the network immediately.
            if app_state.has_identity() {
                if let Err(e) = rt_handle.block_on(async { app_state.start_network().await }) {
                    tracing::warn!("Failed to start network on launch: {e}");
                }
            }

            app.manage(std::sync::Mutex::new(app_state));
            app.manage(rt_handle.clone());

            // Spawn the network listener on a dedicated thread.
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                rt.block_on(state::run_network_listener(app_handle));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Identity
            commands::create_identity,
            commands::get_own_profile,
            commands::update_profile,
            commands::export_identity,
            commands::import_identity,
            commands::has_identity,
            // Posts
            commands::create_post,
            commands::create_reply,
            commands::react_to_post,
            commands::repost,
            // Feeds
            commands::get_following_feed,
            commands::get_discover_feed,
            commands::get_new_voices_feed,
            commands::get_thread,
            // Social
            commands::follow_user,
            commands::unfollow_user,
            commands::block_user,
            commands::get_profile,
            commands::get_following_list,
            // Network
            commands::start_network,
            commands::get_network_status,
            commands::get_storage_stats,
            commands::set_cache_limit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Murmur");
}
