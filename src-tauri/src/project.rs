//! Directory-based project management.
//!
//! A "project" is just a folder on disk with the following layout:
//!
//! ```text
//! my-book/
//! ├── bookforge.json      # project metadata
//! ├── chapters/
//! │   ├── 01-introduction.md
//! │   ├── 02-chapter-one.md
//! │   └── ...
//! ├── assets/
//! │   └── cover.png
//! └── exports/            # written by export commands
//! ```
//!
//! All operations are filesystem-backed so users can sync via git, Dropbox, etc.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub title: String,
    pub author: String,
    pub language: String,
    pub description: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            title: "Untitled Book".into(),
            author: "Anonymous".into(),
            language: "en".into(),
            description: String::new(),
            version: "0.1.0".into(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: String,
    pub filename: String,
    pub title: String,
    pub order: u32,
    pub word_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTree {
    pub root: String,
    pub meta: ProjectMeta,
    pub chapters: Vec<Chapter>,
    pub assets: Vec<String>,
}

const META_FILE: &str = "bookforge.json";
const CHAPTERS_DIR: &str = "chapters";
const ASSETS_DIR: &str = "assets";
const EXPORTS_DIR: &str = "exports";
const BIBLE_DIR: &str = "bible";
const STATS_FILE: &str = "stats.json";

pub fn meta_path(root: &Path) -> PathBuf {
    root.join(META_FILE)
}

pub fn chapters_dir(root: &Path) -> PathBuf {
    root.join(CHAPTERS_DIR)
}

pub fn assets_dir(root: &Path) -> PathBuf {
    root.join(ASSETS_DIR)
}

pub fn exports_dir(root: &Path) -> PathBuf {
    root.join(EXPORTS_DIR)
}

pub fn bible_dir(root: &Path) -> PathBuf {
    root.join(BIBLE_DIR)
}

pub fn stats_path(root: &Path) -> PathBuf {
    root.join(STATS_FILE)
}

/// Create a new project at `root`. Fails if the directory already contains a
/// `bookforge.json`.
pub fn create_project(root: &Path, meta: ProjectMeta) -> Result<ProjectTree, String> {
    if meta_path(root).exists() {
        return Err(format!(
            "Project already exists at {}",
            root.display()
        ));
    }
    fs::create_dir_all(root).map_err(|e| format!("mkdir {}: {}", root.display(), e))?;
    fs::create_dir_all(chapters_dir(root))
        .map_err(|e| format!("mkdir chapters: {}", e))?;
    fs::create_dir_all(assets_dir(root)).map_err(|e| format!("mkdir assets: {}", e))?;
    fs::create_dir_all(exports_dir(root)).map_err(|e| format!("mkdir exports: {}", e))?;
    fs::create_dir_all(bible_dir(root)).map_err(|e| format!("mkdir bible: {}", e))?;

    write_meta(root, &meta)?;
    Ok(ProjectTree {
        root: root.display().to_string(),
        meta,
        chapters: Vec::new(),
        assets: Vec::new(),
    })
}

/// Open an existing project at `root`.
pub fn open_project(root: &Path) -> Result<ProjectTree, String> {
    if !meta_path(root).exists() {
        return Err(format!(
            "Not a BookForge project (missing {}): {}",
            META_FILE,
            root.display()
        ));
    }
    let meta_str =
        fs::read_to_string(meta_path(root)).map_err(|e| format!("read meta: {}", e))?;
    let meta: ProjectMeta =
        serde_json::from_str(&meta_str).map_err(|e| format!("parse meta: {}", e))?;

    let mut chapters = list_chapters(root)?;
    chapters.sort_by_key(|c| c.order);

    let assets = list_assets(root)?;

    Ok(ProjectTree {
        root: root.display().to_string(),
        meta,
        chapters,
        assets,
    })
}

pub fn write_meta(root: &Path, meta: &ProjectMeta) -> Result<(), String> {
    let mut meta = meta.clone();
    meta.updated_at = chrono::Utc::now().to_rfc3339();
    let s = serde_json::to_string_pretty(&meta).map_err(|e| format!("serialize meta: {}", e))?;
    fs::write(meta_path(root), s).map_err(|e| format!("write meta: {}", e))?;
    Ok(())
}

pub fn list_chapters(root: &Path) -> Result<Vec<Chapter>, String> {
    let dir = chapters_dir(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {}", dir.display(), e))? {
        let entry = entry.map_err(|e| format!("dir entry: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let title = derive_title(&path, &filename);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let word_count = content.split_whitespace().count() as u64;
        let order = derive_order(&filename);
        let id = filename.trim_end_matches(".md").to_string();
        out.push(Chapter {
            id,
            filename,
            title,
            order,
            word_count,
        });
    }
    Ok(out)
}

pub fn list_assets(root: &Path) -> Result<Vec<String>, String> {
    let dir = assets_dir(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("read_dir assets: {}", e))? {
        let entry = entry.map_err(|e| format!("dir entry: {}", e))?;
        if entry.path().is_file() {
            out.push(
                entry
                    .path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string(),
            );
        }
    }
    out.sort();
    Ok(out)
}

fn derive_title(path: &Path, filename: &str) -> String {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("# ") {
                return t.trim_start_matches("# ").to_string();
            }
        }
    }
    filename
        .trim_end_matches(".md")
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '-' || c == '_')
        .replace(['-', '_'], " ")
        .to_case_a_like()
}

fn derive_order(filename: &str) -> u32 {
    let prefix: String = filename
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    prefix.parse::<u32>().unwrap_or(999)
}

pub fn create_chapter(root: &Path, filename: &str, title: &str) -> Result<Chapter, String> {
    let path = chapters_dir(root).join(filename);
    if path.exists() {
        return Err(format!("Chapter already exists: {}", filename));
    }
    let body = format!("# {}\n\n", title);
    fs::write(&path, body).map_err(|e| format!("write chapter: {}", e))?;
    Ok(Chapter {
        id: filename.trim_end_matches(".md").to_string(),
        filename: filename.to_string(),
        title: title.to_string(),
        order: derive_order(filename),
        word_count: 0,
    })
}

pub fn read_chapter(root: &Path, filename: &str) -> Result<String, String> {
    let path = chapters_dir(root).join(filename);
    fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))
}

pub fn write_chapter(root: &Path, filename: &str, content: &str) -> Result<u64, String> {
    let path = chapters_dir(root).join(filename);
    // Compute the delta in word count vs. the previous version so we can
    // record progress for streak tracking.
    let prev_words = fs::read_to_string(&path)
        .map(|s| s.split_whitespace().count() as u64)
        .unwrap_or(0);
    let new_words = content.split_whitespace().count() as u64;
    let delta = new_words.saturating_sub(prev_words);

    fs::write(&path, content).map_err(|e| format!("write {}: {}", path.display(), e))?;

    if let Ok(tree) = open_project(root) {
        let _ = write_meta(root, &tree.meta);
    }

    // Record the writing delta for streak tracking. The minutes_spent is
    // approximate — the frontend will send a more accurate value via
    // record_session if it tracks the session timer, but we still bump
    // today's word count immediately so the daily goal updates live.
    if delta > 0 {
        let _ = record_session(root, delta, 0);
    }

    Ok(new_words)
}

pub fn delete_chapter(root: &Path, filename: &str) -> Result<(), String> {
    let path = chapters_dir(root).join(filename);
    if !path.exists() {
        return Err(format!("Chapter not found: {}", filename));
    }
    fs::remove_file(&path).map_err(|e| format!("remove {}: {}", path.display(), e))
}

pub fn rename_chapter(root: &Path, old: &str, new: &str) -> Result<(), String> {
    let from = chapters_dir(root).join(old);
    let to = chapters_dir(root).join(new);
    if !from.exists() {
        return Err(format!("Chapter not found: {}", old));
    }
    if to.exists() {
        return Err(format!("Chapter already exists: {}", new));
    }
    fs::rename(&from, &to).map_err(|e| format!("rename: {}", e))
}

/// Quick & dirty title-case helper.
trait ToCaseALike {
    fn to_case_a_like(self) -> String;
}
impl ToCaseALike for String {
    fn to_case_a_like(self) -> String {
        let mut out = String::new();
        let mut up = true;
        for ch in self.chars() {
            if ch.is_whitespace() || ch == '-' || ch == '_' {
                out.push(' ');
                up = true;
            } else if up {
                for u in ch.to_uppercase() {
                    out.push(u);
                }
                up = false;
            } else {
                out.push(ch);
            }
        }
        out
    }
}

/// Returns the default workspace directory: `~/.bookforge`
pub fn default_workspace_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".bookforge")
}

/// Ensures the `~/.bookforge` directory exists.
pub fn ensure_workspace_dir() -> Result<PathBuf, String> {
    let dir = default_workspace_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create workspace directory {}: {}", dir.display(), e))?;
    }
    Ok(dir)
}

/// Creates a new book project inside `~/.bookforge` given only a name (and optional author).
pub fn create_book_in_workspace(name: &str, author: Option<&str>) -> Result<ProjectTree, String> {
    let base_dir = ensure_workspace_dir()?;
    let name_trimmed = name.trim();
    if name_trimmed.is_empty() {
        return Err("Book name cannot be empty.".to_string());
    }

    // Convert name to safe directory slug: e.g. "My Great Book" -> "my-great-book"
    let mut slug = String::new();
    let mut last_was_dash = false;
    for c in name_trimmed.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    let slug = if slug.is_empty() {
        "untitled-book".to_string()
    } else {
        slug
    };

    // Avoid collision if folder already exists
    let mut target_dir = base_dir.join(&slug);
    let mut counter = 2;
    while target_dir.exists() {
        target_dir = base_dir.join(format!("{}-{}", slug, counter));
        counter += 1;
    }

    let now = chrono::Utc::now().to_rfc3339();
    let author_val = match author {
        Some(a) if !a.trim().is_empty() => a.trim().to_string(),
        _ => "Anonymous".to_string(),
    };

    let meta = ProjectMeta {
        title: name_trimmed.to_string(),
        author: author_val,
        language: "en".to_string(),
        description: String::new(),
        version: "0.1.0".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    let mut tree = create_project(&target_dir, meta)?;

    // Automatically create the first chapter so the book is ready to write!
    let first_ch = create_chapter(&target_dir, "01-chapter.md", "Chapter 1")?;
    tree.chapters.push(first_ch);

    Ok(tree)
}

/// Lists all book projects found directly in `~/.bookforge`.
pub fn list_workspace_books() -> Result<Vec<ProjectTree>, String> {
    let base_dir = ensure_workspace_dir()?;
    let mut books = Vec::new();
    if let Ok(entries) = fs::read_dir(&base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && meta_path(&path).exists() {
                if let Ok(tree) = open_project(&path) {
                    books.push(tree);
                }
            }
        }
    }
    // Sort recently updated first
    books.sort_by(|a, b| b.meta.updated_at.cmp(&a.meta.updated_at));
    Ok(books)
}

// ============================================
// Story Bible — Characters, Locations, Notes
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    pub id: String,
    pub name: String,
    pub role: String,
    pub description: String,
    pub traits: String,
    pub backstory: String,
    pub appearance: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mood: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryNote {
    pub id: String,
    #[serde(rename = "type")]
    pub note_type: String,
    pub title: String,
    pub content: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryBible {
    pub characters: Vec<Character>,
    pub locations: Vec<Location>,
    pub notes: Vec<StoryNote>,
}

fn bible_characters_path(root: &Path) -> PathBuf {
    bible_dir(root).join("characters.json")
}
fn bible_locations_path(root: &Path) -> PathBuf {
    bible_dir(root).join("locations.json")
}
fn bible_notes_path(root: &Path) -> PathBuf {
    bible_dir(root).join("notes.json")
}

fn read_json_or_default<T: serde::de::DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    let s = fs::read_to_string(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    if s.trim().is_empty() {
        return Ok(T::default());
    }
    serde_json::from_str(&s).map_err(|e| format!("parse {}: {}", path.display(), e))
}

fn write_json<T: Serialize>(path: &Path, v: &T) -> Result<(), String> {
    let s = serde_json::to_string_pretty(v).map_err(|e| format!("serialize: {}", e))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir parent: {}", e))?;
    }
    fs::write(path, s).map_err(|e| format!("write {}: {}", path.display(), e))
}

pub fn list_bible(root: &Path) -> Result<StoryBible, String> {
    let characters = read_json_or_default::<Vec<Character>>(&bible_characters_path(root))?;
    let locations = read_json_or_default::<Vec<Location>>(&bible_locations_path(root))?;
    let notes = read_json_or_default::<Vec<StoryNote>>(&bible_notes_path(root))?;
    Ok(StoryBible { characters, locations, notes })
}

pub fn upsert_character(root: &Path, mut ch: Character) -> Result<Character, String> {
    let mut list = read_json_or_default::<Vec<Character>>(&bible_characters_path(root))?;
    if ch.id.is_empty() {
        ch.id = gen_id();
    }
    if let Some(existing) = list.iter_mut().find(|c| c.id == ch.id) {
        *existing = ch.clone();
    } else {
        list.push(ch.clone());
    }
    write_json(&bible_characters_path(root), &list)?;
    Ok(ch)
}

pub fn delete_character(root: &Path, id: &str) -> Result<(), String> {
    let mut list = read_json_or_default::<Vec<Character>>(&bible_characters_path(root))?;
    list.retain(|c| c.id != id);
    write_json(&bible_characters_path(root), &list)
}

pub fn upsert_location(root: &Path, mut loc: Location) -> Result<Location, String> {
    let mut list = read_json_or_default::<Vec<Location>>(&bible_locations_path(root))?;
    if loc.id.is_empty() {
        loc.id = gen_id();
    }
    if let Some(existing) = list.iter_mut().find(|l| l.id == loc.id) {
        *existing = loc.clone();
    } else {
        list.push(loc.clone());
    }
    write_json(&bible_locations_path(root), &list)?;
    Ok(loc)
}

pub fn delete_location(root: &Path, id: &str) -> Result<(), String> {
    let mut list = read_json_or_default::<Vec<Location>>(&bible_locations_path(root))?;
    list.retain(|l| l.id != id);
    write_json(&bible_locations_path(root), &list)
}

pub fn upsert_note(root: &Path, mut note: StoryNote) -> Result<StoryNote, String> {
    let mut list = read_json_or_default::<Vec<StoryNote>>(&bible_notes_path(root))?;
    if note.id.is_empty() {
        note.id = gen_id();
    }
    if let Some(existing) = list.iter_mut().find(|n| n.id == note.id) {
        *existing = note.clone();
    } else {
        list.push(note.clone());
    }
    write_json(&bible_notes_path(root), &list)?;
    Ok(note)
}

pub fn delete_note(root: &Path, id: &str) -> Result<(), String> {
    let mut list = read_json_or_default::<Vec<StoryNote>>(&bible_notes_path(root))?;
    list.retain(|n| n.id != id);
    write_json(&bible_notes_path(root), &list)
}

fn gen_id() -> String {
    // Short, sortable-ish id (timestamp + random) — no extra deps.
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let rand: u32 = rand_like();
    format!("bf{:x}{:04x}", ts, rand)
}

// Tiny in-process RNG (no extra crate needed; not crypto-secure but adequate for ids).
fn rand_like() -> u32 {
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u32> = Cell::new(0x1234_5678);
    }
    SEED.with(|s| {
        let mut x = s.get();
        // xorshift32
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        s.set(x);
        x
    })
}

// ============================================
// Writing progress (stats.json + daily tracking)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyProgress {
    pub date: String,         // YYYY-MM-DD
    pub words_written: u64,
    pub goal_met: bool,
    pub minutes_spent: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WritingStatsFile {
    pub daily: Vec<DailyProgress>,
    pub last_written_at: Option<String>,
    pub current_streak: u32,
    pub longest_streak: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WritingStats {
    pub total_words: u64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub daily_goal: u32,
    pub today_words: u64,
    pub today_goal_met: bool,
    pub goals_met_count: u32,
    pub daily: Vec<DailyProgress>,
}

pub fn read_stats(root: &Path) -> Result<WritingStatsFile, String> {
    read_json_or_default::<WritingStatsFile>(&stats_path(root))
}

pub fn write_stats(root: &Path, stats: &WritingStatsFile) -> Result<(), String> {
    write_json(&stats_path(root), stats)
}

/// Records a writing session: bumps today's word count by `words_written`,
/// increments minutes spent, recomputes the streak.
pub fn record_session(root: &Path, words_written: u64, minutes_spent: u64) -> Result<(), String> {
    let mut stats = read_stats(root)?;
    let today = today_iso();

    let goal = book_daily_goal(root).unwrap_or(500);

    let entry = stats
        .daily
        .iter_mut()
        .find(|d| d.date == today);

    if let Some(e) = entry {
        e.words_written += words_written;
        e.minutes_spent += minutes_spent;
        if e.words_written >= goal as u64 {
            e.goal_met = true;
        }
    } else {
        let words = words_written;
        stats.daily.push(DailyProgress {
            date: today.clone(),
            words_written: words,
            goal_met: words >= goal as u64,
            minutes_spent,
        });
    }

    stats.last_written_at = Some(chrono::Utc::now().to_rfc3339());
    stats.current_streak = compute_streak(&stats.daily);
    stats.longest_streak = stats.longest_streak.max(stats.current_streak);

    write_stats(root, &stats)
}

pub fn writing_stats_summary(root: &Path) -> Result<WritingStats, String> {
    let stats = read_stats(root)?;
    let today = today_iso();
    let today_entry = stats.daily.iter().find(|d| d.date == today);
    let today_words = today_entry.map(|d| d.words_written).unwrap_or(0);
    let today_goal_met = today_entry.map(|d| d.goal_met).unwrap_or(false);
    let goal = book_daily_goal(root).unwrap_or(500);

    let total_words: u64 = book_total_words(root)?;

    // Trim daily to last 90 days for the UI.
    let mut daily_90: Vec<DailyProgress> = stats
        .daily
        .iter()
        .rev()
        .take(90)
        .cloned()
        .collect();
    daily_90.reverse();

    Ok(WritingStats {
        total_words,
        current_streak: stats.current_streak,
        longest_streak: stats.longest_streak,
        daily_goal: goal,
        today_words,
        today_goal_met,
        goals_met_count: stats.daily.iter().filter(|d| d.goal_met).count() as u32,
        daily: daily_90,
    })
}

fn compute_streak(daily: &[DailyProgress]) -> u32 {
    let mut dates_met: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for d in daily {
        if d.goal_met {
            dates_met.insert(d.date.as_str());
        }
    }
    let mut streak = 0u32;
    let today = today_iso();
    let today_d = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").ok();
    if today_d.is_none() {
        return 0;
    }
    let mut cursor = today_d.unwrap();
    let mut skipped_today = false;
    for _ in 0..365 {
        let iso = cursor.format("%Y-%m-%d").to_string();
        if dates_met.contains(iso.as_str()) {
            streak += 1;
        } else if !skipped_today && iso == today {
            skipped_today = true;
            // Allow today to be empty without breaking the streak.
        } else {
            break;
        }
        cursor = cursor.pred_opt().unwrap_or(cursor);
    }
    streak
}

fn today_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

fn book_daily_goal(root: &Path) -> Option<u32> {
    // The daily goal lives in bookforge.json as meta.dailyGoal (optional).
    let raw = fs::read_to_string(meta_path(root)).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get("dailyGoal")
        .and_then(|n| n.as_u64())
        .map(|n| n as u32)
        .or(Some(500))
}

fn book_total_words(root: &Path) -> Result<u64, String> {
    let chapters = list_chapters(root)?;
    Ok(chapters.iter().map(|c| c.word_count).sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_workspace_dir() {
        let dir = ensure_workspace_dir().expect("should ensure workspace dir");
        assert!(dir.exists());
        assert!(dir.to_string_lossy().contains(".bookforge"));
    }

    #[test]
    fn test_create_book_in_workspace() {
        let book = create_book_in_workspace("Test Book Automation", Some("Author Test"))
            .expect("should create book");
        assert_eq!(book.meta.title, "Test Book Automation");
        assert_eq!(book.meta.author, "Author Test");
        assert_eq!(book.chapters.len(), 1);
        assert_eq!(book.chapters[0].title, "Chapter 1");
        assert!(Path::new(&book.root).exists());

        // Clean up test book
        let _ = fs::remove_dir_all(&book.root);
    }
}
