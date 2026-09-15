// Shared TypeScript types mirroring the Rust DTOs.

export interface ProjectMeta {
  title: string;
  author: string;
  language: string;
  description: string;
  version: string;
  createdAt: string;
  updatedAt: string;
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
