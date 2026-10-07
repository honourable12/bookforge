// Thin typed wrappers over `@tauri-apps/api/core invoke`.
import { invoke } from "@tauri-apps/api/core";
import type {
  Chapter,
  ChatMessage,
  ExportResult,
  GenParams,
  ProjectMeta,
  ProjectTree,
  SpawnOptions,
  StoryBible,
  StoryNote,
  Character,
  Location,
  WritingStats,
  UIPrefs,
  StoryMap,
} from "./types";

export const api = {
  ping: () => invoke<string>("ping"),

  // Model
  modelStatus: () => invoke<string>("model_status"),
  modelLoaded: () => invoke<boolean>("model_loaded"),
  modelLocate: () => invoke<string>("model_locate"),
  modelLoad: () => invoke<string>("model_load"),

  // AI
  aiChat: (requestId: string, messages: ChatMessage[], params?: GenParams) =>
    invoke<string>("ai_chat", { requestId, messages, params }),
  aiContinue: (requestId: string, prefix: string, params?: GenParams, root?: string) =>
    invoke<string>("ai_continue", { requestId, prefix, params, root }),
  aiRewrite: (
    requestId: string,
    selection: string,
    instruction: string,
    params?: GenParams,
    root?: string,
  ) =>
    invoke<string>("ai_rewrite", { requestId, selection, instruction, params, root }),
  aiCancel: (requestId: string) => invoke<void>("ai_cancel", { requestId }),

  // Workspace & Books
  bookCreate: (name: string, author?: string) =>
    invoke<ProjectTree>("book_create", { name, author }),
  workspaceDir: () => invoke<string>("workspace_dir"),
  workspaceListBooks: () => invoke<ProjectTree[]>("workspace_list_books"),

  // Projects
  projectCreate: (root: string, meta: ProjectMeta) =>
    invoke<ProjectTree>("project_create", { root, meta }),
  projectOpen: (root: string) => invoke<ProjectTree>("project_open", { root }),
  projectSaveMeta: (root: string, meta: ProjectMeta) =>
    invoke<void>("project_save_meta", { root, meta }),
  projectListRecent: () => invoke<string[]>("project_list_recent"),

  // Chapters
  chapterList: (root: string) => invoke<Chapter[]>("chapter_list", { root }),
  chapterCreate: (root: string, filename: string, title: string) =>
    invoke<Chapter>("chapter_create", { root, filename, title }),
  chapterRead: (root: string, filename: string) =>
    invoke<string>("chapter_read", { root, filename }),
  chapterWrite: (root: string, filename: string, content: string) =>
    invoke<number>("chapter_write", { root, filename, content }),
  chapterDelete: (root: string, filename: string) =>
    invoke<void>("chapter_delete", { root, filename }),
  chapterRename: (root: string, old: string, name: string) =>
    invoke<void>("chapter_rename", { root, old, name }),

  // Export
  exportHtml: (root: string) => invoke<ExportResult>("export_html", { root }),
  exportPdf: (root: string) => invoke<ExportResult>("export_pdf", { root }),
  exportEpub: (root: string) => invoke<ExportResult>("export_epub", { root }),

  // Terminal
  termSpawn: (opts: SpawnOptions) => invoke<string>("term_spawn", { opts }),
  termWrite: (sessionId: string, data: string) =>
    invoke<void>("term_write", { sessionId, data }),
  termResize: (sessionId: string, cols: number, rows: number) =>
    invoke<void>("term_resize", { sessionId, cols, rows }),
  termKill: (sessionId: string) => invoke<void>("term_kill", { sessionId }),

  // ----- New in v0.2: Story Bible -----
  bibleList: (root: string) => invoke<StoryBible>("bible_list", { root }),
  bibleUpsertCharacter: (root: string, character: Character) =>
    invoke<Character>("bible_upsert_character", { root, character }),
  bibleDeleteCharacter: (root: string, id: string) =>
    invoke<void>("bible_delete_character", { root, id }),
  bibleUpsertLocation: (root: string, location: Location) =>
    invoke<Location>("bible_upsert_location", { root, location }),
  bibleDeleteLocation: (root: string, id: string) =>
    invoke<void>("bible_delete_location", { root, id }),
  bibleUpsertNote: (root: string, note: StoryNote) =>
    invoke<StoryNote>("bible_upsert_note", { root, note }),
  bibleDeleteNote: (root: string, id: string) =>
    invoke<void>("bible_delete_note", { root, id }),

  // ----- New in v0.2.1: Story Map -----
  storyMapGet: (root: string) => invoke<StoryMap>("story_map_get", { root }),
  storyMapSave: (root: string, map: StoryMap) =>
    invoke<void>("story_map_save", { root, map }),
  storyMapPreviewContext: (root: string) =>
    invoke<string>("story_map_preview_context", { root }),

  // ----- New in v0.2: Writing stats -----
  writingStats: (root: string) => invoke<WritingStats>("writing_stats", { root }),
  recordSession: (root: string, wordsWritten: number, minutesSpent: number) =>
    invoke<void>("record_session", { root, wordsWritten, minutesSpent }),

  // ----- New in v0.2: UI preferences (persisted via tauri-plugin-store) -----
  prefsLoad: () => invoke<UIPrefs>("prefs_load"),
  prefsSave: (prefs: UIPrefs) => invoke<void>("prefs_save", { prefs }),
};
