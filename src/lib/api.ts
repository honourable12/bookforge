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
  aiContinue: (requestId: string, prefix: string, params?: GenParams) =>
    invoke<string>("ai_continue", { requestId, prefix, params }),
  aiRewrite: (
    requestId: string,
    selection: string,
    instruction: string,
    params?: GenParams,
  ) =>
    invoke<string>("ai_rewrite", { requestId, selection, instruction, params }),
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
};
