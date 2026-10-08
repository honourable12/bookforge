//! Tauri command bindings — the bridge between the Svelte frontend and the
//! Rust backend modules.

use std::path::PathBuf;

use tauri::{AppHandle, State, Window};

use crate::ai::{self, ChatMessage, GenParams, StreamChunk};
use crate::export::{self, ExportResult};
use crate::model;
use crate::project::{
    self, Character, Chapter, Location, ProjectMeta, ProjectTree, StoryBible, StoryNote, StoryMap, MapNode, MapEdge, MapNote, WritingStats,
};
use crate::terminal::{self, SpawnOptions};
use crate::prefs;

// ---------------------------------------------------------------------------
// Model / AI
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn model_status() -> String {
    model::model_status()
}

#[tauri::command]
pub fn model_loaded() -> bool {
    model::is_model_loaded()
}

#[tauri::command]
pub fn model_locate() -> Result<String, String> {
    let p = model::locate_model_file()?;
    Ok(p.display().to_string())
}

#[tauri::command]
pub fn model_load() -> Result<String, String> {
    let _ = model::get_model()?;
    Ok(model::model_status())
}

#[tauri::command]
pub async fn ai_chat(
    app: AppHandle,
    window: Window,
    request_id: String,
    messages: Vec<ChatMessage>,
    params: Option<GenParams>,
    root: Option<String>,
) -> Result<String, String> {
    // If the caller passed a project root, load the Story Map and inject it
    // as context so the LLM knows who's who and how they relate — for
    // chat, write-to-book, and every other assistant interaction.
    let story_map_context = match root {
        Some(r) => project::story_map_as_context(&PathBuf::from(r)),
        None => String::new(),
    };
    let prompt = ai::build_chat_prompt(&messages, &story_map_context);
    let params = params.unwrap_or_default();
    let result = tokio::task::spawn_blocking(move || {
        ai::run_completion(&app, &window, "ai_chat_chunk", &request_id, &prompt, &params)
    })
    .await
    .map_err(|e| format!("join error: {}", e))??;
    Ok(result)
}

#[tauri::command]
pub async fn ai_continue(
    app: AppHandle,
    window: Window,
    request_id: String,
    prefix: String,
    params: Option<GenParams>,
    root: Option<String>,
) -> Result<String, String> {
    // If the caller passed a project root, load the Story Map and inject it
    // as context so the LLM knows who's who and how they relate.
    let story_map_context = match root {
        Some(r) => project::story_map_as_context(&PathBuf::from(r)),
        None => String::new(),
    };
    let prompt = ai::build_continue_prompt(&prefix, &story_map_context);
    let params = params.unwrap_or_default();
    let result = tokio::task::spawn_blocking(move || {
        ai::run_completion(&app, &window, "ai_continue_chunk", &request_id, &prompt, &params)
    })
    .await
    .map_err(|e| format!("join error: {}", e))??;
    Ok(result)
}

#[tauri::command]
pub async fn ai_rewrite(
    app: AppHandle,
    window: Window,
    request_id: String,
    selection: String,
    instruction: String,
    params: Option<GenParams>,
    root: Option<String>,
) -> Result<String, String> {
    let story_map_context = match root {
        Some(r) => project::story_map_as_context(&PathBuf::from(r)),
        None => String::new(),
    };
    let prompt = ai::build_rewrite_prompt(&selection, &instruction, &story_map_context);
    let params = params.unwrap_or_default();
    let result = tokio::task::spawn_blocking(move || {
        ai::run_completion(&app, &window, "ai_rewrite_chunk", &request_id, &prompt, &params)
    })
    .await
    .map_err(|e| format!("join error: {}", e))??;
    Ok(result)
}

#[tauri::command]
pub fn ai_cancel(_request_id: String) -> Result<(), String> {
    Ok(())
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn book_create(name: String, author: Option<String>) -> Result<ProjectTree, String> {
    project::create_book_in_workspace(&name, author.as_deref())
}

#[tauri::command]
pub fn workspace_dir() -> Result<String, String> {
    let dir = project::ensure_workspace_dir()?;
    Ok(dir.display().to_string())
}

#[tauri::command]
pub fn workspace_list_books() -> Result<Vec<ProjectTree>, String> {
    project::list_workspace_books()
}

#[tauri::command]
pub fn project_create(root: String, meta: ProjectMeta) -> Result<ProjectTree, String> {
    project::create_project(&PathBuf::from(root), meta)
}

#[tauri::command]
pub fn project_open(root: String) -> Result<ProjectTree, String> {
    project::open_project(&PathBuf::from(root))
}

#[tauri::command]
pub fn project_list_recent(_state: State<crate::AppState>) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn project_save_meta(root: String, meta: ProjectMeta) -> Result<(), String> {
    project::write_meta(&PathBuf::from(root), &meta)
}

#[tauri::command]
pub fn chapter_list(root: String) -> Result<Vec<Chapter>, String> {
    let tree = project::open_project(&PathBuf::from(root))?;
    let mut chapters = tree.chapters;
    chapters.sort_by_key(|c| c.order);
    Ok(chapters)
}

#[tauri::command]
pub fn chapter_create(root: String, filename: String, title: String) -> Result<Chapter, String> {
    project::create_chapter(&PathBuf::from(root), &filename, &title)
}

#[tauri::command]
pub fn chapter_read(root: String, filename: String) -> Result<String, String> {
    project::read_chapter(&PathBuf::from(root), &filename)
}

#[tauri::command]
pub fn chapter_write(root: String, filename: String, content: String) -> Result<u64, String> {
    project::write_chapter(&PathBuf::from(root), &filename, &content)
}

#[tauri::command]
pub fn chapter_delete(root: String, filename: String) -> Result<(), String> {
    project::delete_chapter(&PathBuf::from(root), &filename)
}

#[tauri::command]
pub fn chapter_rename(root: String, old: String, new: String) -> Result<(), String> {
    project::rename_chapter(&PathBuf::from(root), &old, &new)
}

// ---------------------------------------------------------------------------
// Story Bible — characters, locations, notes (new in v0.2)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn bible_list(root: String) -> Result<StoryBible, String> {
    project::list_bible(&PathBuf::from(root))
}

#[tauri::command]
pub fn bible_upsert_character(root: String, character: Character) -> Result<Character, String> {
    project::upsert_character(&PathBuf::from(root), character)
}

#[tauri::command]
pub fn bible_delete_character(root: String, id: String) -> Result<(), String> {
    project::delete_character(&PathBuf::from(root), &id)
}

#[tauri::command]
pub fn bible_upsert_location(root: String, location: Location) -> Result<Location, String> {
    project::upsert_location(&PathBuf::from(root), location)
}

#[tauri::command]
pub fn bible_delete_location(root: String, id: String) -> Result<(), String> {
    project::delete_location(&PathBuf::from(root), &id)
}

#[tauri::command]
pub fn bible_upsert_note(root: String, note: StoryNote) -> Result<StoryNote, String> {
    project::upsert_note(&PathBuf::from(root), note)
}

#[tauri::command]
pub fn bible_delete_note(root: String, id: String) -> Result<(), String> {
    project::delete_note(&PathBuf::from(root), &id)
}

// ---------------------------------------------------------------------------
// Story Map — graph of relationships (new in v0.2.1)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn story_map_get(root: String) -> Result<StoryMap, String> {
    project::read_story_map(&PathBuf::from(root))
}

#[tauri::command]
pub fn story_map_save(root: String, map: StoryMap) -> Result<(), String> {
    project::write_story_map(&PathBuf::from(root), &map)
}

#[tauri::command]
pub fn story_map_preview_context(root: String) -> Result<String, String> {
    Ok(project::story_map_as_context(&PathBuf::from(root)))
}

// ---------------------------------------------------------------------------
// Writing stats (new in v0.2)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn writing_stats(root: String) -> Result<WritingStats, String> {
    project::writing_stats_summary(&PathBuf::from(root))
}

#[tauri::command]
pub fn record_session(
    root: String,
    words_written: u64,
    minutes_spent: u64,
) -> Result<(), String> {
    project::record_session(&PathBuf::from(root), words_written, minutes_spent)
}

// ---------------------------------------------------------------------------
// UI preferences (new in v0.2 — persisted via tauri-plugin-store)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn prefs_load(app: AppHandle) -> Result<prefs::UIPrefs, String> {
    prefs::load(&app)
}

#[tauri::command]
pub fn prefs_save(app: AppHandle, prefs: prefs::UIPrefs) -> Result<(), String> {
    prefs::save(&app, &prefs)
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn export_html(root: String) -> Result<ExportResult, String> {
    export::export_html(&PathBuf::from(root))
}

#[tauri::command]
pub fn export_pdf(root: String) -> Result<ExportResult, String> {
    export::export_pdf(&PathBuf::from(root))
}

#[tauri::command]
pub fn export_epub(root: String) -> Result<ExportResult, String> {
    export::export_epub(&PathBuf::from(root))
}

// ---------------------------------------------------------------------------
// Terminal
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn term_spawn(app: AppHandle, opts: SpawnOptions) -> Result<String, String> {
    terminal::spawn_session(app, opts)
}

#[tauri::command]
pub fn term_write(session_id: String, data: String) -> Result<(), String> {
    terminal::write_session(&session_id, data.as_bytes())
}

#[tauri::command]
pub fn term_resize(session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    terminal::resize_session(&session_id, cols, rows)
}

#[tauri::command]
pub fn term_kill(session_id: String) -> Result<(), String> {
    terminal::kill_session(&session_id)
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn ping() -> String {
    "pong".into()
}

#[tauri::command]
pub fn default_shell() -> String {
    terminal::default_shell()
}

#[tauri::command]
pub fn stream_chunk_demo() -> StreamChunk {
    StreamChunk {
        request_id: "demo".into(),
        token: "".into(),
        finish: true,
        error: None,
    }
}
