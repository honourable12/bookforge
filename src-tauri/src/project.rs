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
    fs::write(&path, content).map_err(|e| format!("write {}: {}", path.display(), e))?;
    let words = content.split_whitespace().count() as u64;
    if let Ok(tree) = open_project(root) {
        let _ = write_meta(root, &tree.meta);
    }
    Ok(words)
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
