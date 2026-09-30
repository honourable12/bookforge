//! UI preferences persistence — uses `tauri-plugin-store` to persist
//! user UI choices (theme, focus mode, font size, etc.) across sessions.
//!
//! The store lives at `<app_data_dir>/bookforge-ui.json` (default location
//! for `tauri-plugin-store` on the default builder).

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::{Store, StoreExt};

const STORE_FILE: &str = "bookforge-ui.json";
const PREFS_KEY: &str = "prefs";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIPrefs {
    pub theme: String,         // "sepia" | "light" | "dark" | "night"
    pub focus_mode: bool,
    pub sidebar_collapsed: bool,
    pub ai_panel_open: bool,
    pub font_size: u32,
    pub typewriter_mode: bool,
    pub show_terminal: bool,
}

impl Default for UIPrefs {
    fn default() -> Self {
        Self {
            theme: "sepia".to_string(),
            focus_mode: false,
            sidebar_collapsed: false,
            ai_panel_open: false,
            font_size: 18,
            typewriter_mode: false,
            show_terminal: false,
        }
    }
}

fn open_store(app: &AppHandle) -> Result<std::sync::Arc<Store<tauri::Wry>>, String> {
    app.store(STORE_FILE)
        .map_err(|e| format!("Failed to open preferences store: {}", e))
}

pub fn load(app: &AppHandle) -> Result<UIPrefs, String> {
    let store = open_store(app)?;
    let raw = store.get(PREFS_KEY);
    let Some(raw) = raw else {
        return Ok(UIPrefs::default());
    };
    // tauri-plugin-store v2 returns `serde_json::Value` directly.
    match serde_json::from_value::<UIPrefs>(raw.clone()) {
        Ok(p) => Ok(p),
        Err(_) => {
            // Try as JSON string for older store versions.
            if let Some(s) = raw.as_str() {
                if let Ok(p) = serde_json::from_str::<UIPrefs>(s) {
                    return Ok(p);
                }
            }
            Ok(UIPrefs::default())
        }
    }
}

pub fn save(app: &AppHandle, prefs: &UIPrefs) -> Result<(), String> {
    let store = open_store(app)?;
    let v = serde_json::to_value(prefs).map_err(|e| format!("serialize prefs: {}", e))?;
    store.set(PREFS_KEY, v);
    store.save().map_err(|e| format!("save prefs: {}", e))?;
    Ok(())
}
