// Reactive Svelte stores for the BookForge app shell.
import { writable, derived, get } from "svelte/store";
import type {
  Chapter,
  ProjectTree,
  ChatMessage,
  GenParams,
  StoryBible,
  WritingStats,
  UIPrefs,
  Theme,
} from "./types";
import { DEFAULT_GEN_PARAMS, DEFAULT_UI_PREFS } from "./types";
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

// ----- New in v0.2: navigation -----
export type View = "library" | "writer" | "bible" | "stats";
export const view = writable<View>("library");
export const activeBibleTab = writable<"characters" | "locations" | "notes">("characters");

// ----- New in v0.2: UI preferences (persisted via backend tauri-plugin-store) -----
export const prefs = writable<UIPrefs>({ ...DEFAULT_UI_PREFS });
export const theme = derived(prefs, ($p) => $p.theme);
export const focusMode = derived(prefs, ($p) => $p.focusMode);
export const sidebarCollapsed = derived(prefs, ($p) => $p.sidebarCollapsed);
export const aiPanelOpen = derived(prefs, ($p) => $p.aiPanelOpen);
export const fontSize = derived(prefs, ($p) => $p.fontSize);
export const typewriterMode = derived(prefs, ($p) => $p.typewriterMode);
export const showTerminal = derived(prefs, ($p) => $p.showTerminal);

let prefsLoaded = false;
export async function loadPrefs() {
  if (prefsLoaded) return;
  try {
    const p = await api.prefsLoad();
    prefs.set({ ...DEFAULT_UI_PREFS, ...p });
    applyTheme(get(prefs).theme);
    prefsLoaded = true;
  } catch (e) {
    // Backend may not support prefs_load on older versions; fall back to defaults.
    console.warn("prefs_load unavailable, using defaults:", e);
    applyTheme(get(prefs).theme);
    prefsLoaded = true;
  }
}

let prefsSaveTimer: ReturnType<typeof setTimeout> | null = null;
export function updatePrefs(patch: Partial<UIPrefs>) {
  prefs.update((p) => {
    const next = { ...p, ...patch };
    // Apply theme immediately if it changed.
    if (patch.theme && patch.theme !== p.theme) {
      applyTheme(patch.theme);
    }
    // Debounce save to backend.
    if (prefsSaveTimer) clearTimeout(prefsSaveTimer);
    prefsSaveTimer = setTimeout(async () => {
      try {
        await api.prefsSave(next);
      } catch (e) {
        console.warn("prefs_save failed:", e);
      }
    }, 250);
    return next;
  });
}

export function toggleFocusMode() {
  updatePrefs({ focusMode: !get(prefs).focusMode });
}
export function toggleSidebar() {
  updatePrefs({ sidebarCollapsed: !get(prefs).sidebarCollapsed });
}
export function toggleAIPanel() {
  // Opening AI from focus mode also exits focus mode.
  const cur = get(prefs);
  if (!cur.aiPanelOpen && cur.focusMode) {
    updatePrefs({ aiPanelOpen: true, focusMode: false });
  } else {
    updatePrefs({ aiPanelOpen: !cur.aiPanelOpen });
  }
}
export function toggleTypewriter() {
  updatePrefs({ typewriterMode: !get(prefs).typewriterMode });
}
export function toggleTerminal() {
  updatePrefs({ showTerminal: !get(prefs).showTerminal });
}
export function setFontSize(n: number) {
  updatePrefs({ fontSize: Math.max(15, Math.min(24, n)) });
}
export function setTheme(t: Theme) {
  updatePrefs({ theme: t });
}

export function applyTheme(t: Theme) {
  if (typeof document !== "undefined") {
    document.documentElement.setAttribute("data-theme", t);
  }
}

// ----- New in v0.2: Story Bible store -----
export const storyBible = writable<StoryBible>({ characters: [], locations: [], notes: [] });

export async function refreshStoryBible() {
  const p = get(project);
  if (!p) {
    storyBible.set({ characters: [], locations: [], notes: [] });
    return;
  }
  try {
    const b = await api.bibleList(p.root);
    storyBible.set(b);
  } catch (e) {
    console.warn("bible_list failed:", e);
    storyBible.set({ characters: [], locations: [], notes: [] });
  }
}

// ----- New in v0.2: Writing stats store -----
export const writingStats = writable<WritingStats | null>(null);

export async function refreshWritingStats() {
  const p = get(project);
  if (!p) {
    writingStats.set(null);
    return;
  }
  try {
    const s = await api.writingStats(p.root);
    writingStats.set(s);
  } catch (e) {
    console.warn("writing_stats failed:", e);
    writingStats.set(null);
  }
}

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
    await refreshStoryBible();
    await refreshWritingStats();
    view.set("writer");
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

// Apply default theme synchronously on module load (before first render).
// loadPrefs() may override this later if the user has a saved preference.
// This prevents a flash of unstyled content if the Tauri backend isn't
// rebuilt with the new prefs_load command yet.
if (typeof document !== "undefined") {
  document.documentElement.setAttribute("data-theme", "sepia");
}
