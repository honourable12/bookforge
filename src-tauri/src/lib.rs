//! BookForge — Tauri library entrypoint.
//!
//! Wires up all command modules and starts the Tauri runtime.

pub mod ai;
pub mod commands;
pub mod export;
pub mod model;
pub mod prefs;
pub mod project;
pub mod terminal;

use parking_lot::Mutex;

/// Shared application state. Currently a placeholder for things like the
/// "recent projects" list and active session metadata.
pub struct AppState {
    pub recent_projects: Mutex<Vec<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            recent_projects: Mutex::new(Vec::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // misc
            commands::ping,
            commands::stream_chunk_demo,
            commands::default_shell,
            // model
            commands::model_status,
            commands::model_loaded,
            commands::model_locate,
            commands::model_load,
            // ai
            commands::ai_chat,
            commands::ai_continue,
            commands::ai_rewrite,
            commands::ai_cancel,
            // project
            commands::book_create,
            commands::workspace_dir,
            commands::workspace_list_books,
            commands::project_create,
            commands::project_open,
            commands::project_list_recent,
            commands::project_save_meta,
            // chapter
            commands::chapter_list,
            commands::chapter_create,
            commands::chapter_read,
            commands::chapter_write,
            commands::chapter_delete,
            commands::chapter_rename,
            // story bible (new in v0.2)
            commands::bible_list,
            commands::bible_upsert_character,
            commands::bible_delete_character,
            commands::bible_upsert_location,
            commands::bible_delete_location,
            commands::bible_upsert_note,
            commands::bible_delete_note,
            // writing stats (new in v0.2)
            commands::writing_stats,
            commands::record_session,
            // ui prefs (new in v0.2)
            commands::prefs_load,
            commands::prefs_save,
            // export
            commands::export_html,
            commands::export_pdf,
            commands::export_epub,
            // terminal
            commands::term_spawn,
            commands::term_write,
            commands::term_resize,
            commands::term_kill,
        ])
        .setup(|_app| {
            // Automatically initialize the ~/.bookforge directory when the app starts.
            if let Err(e) = project::ensure_workspace_dir() {
                log::error!("Failed to create ~/.bookforge workspace: {}", e);
            } else {
                log::info!("Workspace directory ~/.bookforge initialized.");
            }

            // Pre-warm the model loader on a background thread so the first
            // chat request isn't blocked by ~5s of model loading.
            tauri::async_runtime::spawn(async {
                if model::locate_model_file().is_ok() {
                    log::info!("Model file located, preloading...");
                    let _ = model::get_model();
                } else {
                    log::warn!("Model file not found at startup; AI features will be unavailable until a model is downloaded.");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BookForge");
}
