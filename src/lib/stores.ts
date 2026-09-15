// Reactive Svelte stores for the BookForge app shell.
import { writable, derived, get } from "svelte/store";
import type { Chapter, ProjectTree, ChatMessage, GenParams } from "./types";
import { DEFAULT_GEN_PARAMS } from "./types";
import { api } from "./api";

export const project = writable<ProjectTree | null>(null);
export const activeChapterFilename = writable<string | null>(null);

export const activeChapter = derived(
  [project, activeChapterFilename],
  ([$project, $filename]) => {
    if (!$project || !$filename) return null;
    return $project.chapters.find((c) => c.filename === $filename) ?? null;
  },
);

export const chatMessages = writable<ChatMessage[]>([]);
export const isGenerating = writable(false);
export const activeRequestId = writable<string | null>(null);

export const genParams = writable<GenParams>({ ...DEFAULT_GEN_PARAMS });

export const modelStatus = writable<string>("Not loaded");
export const modelLoaded = writable<boolean>(false);

export const terminals = writable<
  { id: string; title: string; cwd: string }[]
>([]);
export const activeTerminalId = writable<string | null>(null);

export const busy = writable<boolean>(false);
export const toast = writable<{ kind: "info" | "success" | "error"; msg: string } | null>(null);

export function notify(kind: "info" | "success" | "error", msg: string) {
  toast.set({ kind, msg });
  setTimeout(() => {
    if (get(toast)?.msg === msg) toast.set(null);
  }, 3500);
}

export async function refreshModelStatus() {
  try {
    const s = await api.modelStatus();
    const l = await api.modelLoaded();
    modelStatus.set(s);
    modelLoaded.set(l);
  } catch (e) {
    modelStatus.set(`Error: ${e}`);
    modelLoaded.set(false);
  }
}

export const workspaceBooks = writable<ProjectTree[]>([]);
export const workspacePath = writable<string>("");

export async function refreshWorkspaceBooks() {
  try {
    const dir = await api.workspaceDir();
    workspacePath.set(dir);
    const books = await api.workspaceListBooks();
    workspaceBooks.set(books);
  } catch (e) {
    console.error("Failed to load workspace books:", e);
  }
}

export async function createBook(name: string, author?: string) {
  try {
    const tree = await api.bookCreate(name, author);
    project.set(tree);
    if (tree.chapters.length > 0) {
      activeChapterFilename.set(tree.chapters[0].filename);
    } else {
      activeChapterFilename.set(null);
    }
    await refreshWorkspaceBooks();
    notify("success", `Created book "${tree.meta.title}" in ${tree.root}`);
    return tree;
  } catch (e) {
    notify("error", `Failed to create book: ${e}`);
    throw e;
  }
}

export async function openProjectFolder(root: string) {
  try {
    const tree = await api.projectOpen(root);
    project.set(tree);
    if (tree.chapters.length > 0) {
      activeChapterFilename.set(tree.chapters[0].filename);
    } else {
      activeChapterFilename.set(null);
    }
    await refreshWorkspaceBooks();
    notify("success", `Opened "${tree.meta.title}" (${tree.chapters.length} chapters)`);
  } catch (e) {
    notify("error", `Open failed: ${e}`);
  }
}

export async function refreshChapters() {
  const p = get(project);
  if (!p) return;
  const chapters = await api.chapterList(p.root);
  project.set({ ...p, chapters });
}
