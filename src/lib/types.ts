// Shared TypeScript types mirroring the Rust DTOs.

export interface ProjectMeta {
  title: string;
  author: string;
  language: string;
  description: string;
  version: string;
  createdAt: string;
  updatedAt: string;
  // Optional — added by BookForge v0.2 to encourage writing.
  // Older books that lack these fields fall back to defaults.
  coverColor?: string;
  genre?: string;
  dailyGoal?: number;
  currentStreak?: number;
  longestStreak?: number;
  lastWrittenAt?: string | null;
}

export interface Chapter {
  id: string;
  filename: string;
  title: string;
  order: number;
  wordCount: number;
}

export interface ProjectTree {
  root: string;
  meta: ProjectMeta;
  chapters: Chapter[];
  assets: string[];
}

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface GenParams {
  maxTokens: number;
  temperature: number;
  topP: number;
  topK: number;
  repeatPenalty: number;
  seed: number;
}

export const DEFAULT_GEN_PARAMS: GenParams = {
  maxTokens: 512,
  temperature: 0.7,
  topP: 0.9,
  topK: 40,
  repeatPenalty: 1.1,
  seed: 0xcafebabe,
};

export interface StreamChunk {
  requestId: string;
  token: string;
  finish: boolean;
  error: string | null;
}

export interface ExportResult {
  path: string;
  format: string;
  sizeBytes: number;
}

export interface SpawnOptions {
  cmd?: string;
  cwd?: string;
  cols?: number;
  rows?: number;
  env?: Record<string, string>;
}

export interface TerminalOutput {
  sessionId: string;
  data: string;
}

// ----- New in v0.2: Story Bible -----

export interface Character {
  id: string;
  name: string;
  role: string;
  description: string;
  traits: string;
  backstory: string;
  appearance: string;
  color: string;
}

export interface Location {
  id: string;
  name: string;
  description: string;
  mood: string;
  color: string;
}

export type StoryNoteType = "outline" | "worldbuilding" | "plot" | "theme";

export interface StoryNote {
  id: string;
  type: StoryNoteType;
  title: string;
  content: string;
  color: string;
}

export interface StoryBible {
  characters: Character[];
  locations: Location[];
  notes: StoryNote[];
}

// ----- New in v0.2: Writing progress -----

export interface DailyProgress {
  date: string; // YYYY-MM-DD
  wordsWritten: number;
  goalMet: boolean;
  minutesSpent: number;
}

export interface WritingStats {
  totalWords: number;
  currentStreak: number;
  longestStreak: number;
  dailyGoal: number;
  todayWords: number;
  todayGoalMet: boolean;
  goalsMetCount: number;
  daily: DailyProgress[]; // last 90 days
}

// ----- New in v0.2: UI preferences (persisted via tauri-plugin-store) -----

export type Theme = "sepia" | "light" | "dark" | "night";

export interface UIPrefs {
  theme: Theme;
  focusMode: boolean;
  sidebarCollapsed: boolean;
  aiPanelOpen: boolean;
  fontSize: number;
  typewriterMode: boolean;
  showTerminal: boolean;
}

export const DEFAULT_UI_PREFS: UIPrefs = {
  theme: "sepia",
  focusMode: false,
  sidebarCollapsed: false,
  aiPanelOpen: false,
  fontSize: 18,
  typewriterMode: false,
  showTerminal: false,
};
