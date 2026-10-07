<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { EditorView } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import { markdown } from "@codemirror/lang-markdown";
  import { defaultHighlightStyle, syntaxHighlighting } from "@codemirror/language";
  import { dracula } from "thememirror";
  import { api } from "$lib/api";
  import {
    activeChapter,
    activeChapterFilename,
    activeRequestId,
    focusMode,
    fontSize,
    typewriterMode,
    aiPanelOpen,
    showTerminal,
    genParams,
    isGenerating,
    notify,
    project,
    refreshChapters,
    refreshWritingStats,
    toggleFocusMode,
    toggleAIPanel,
    toggleTypewriter,
    setFontSize,
  } from "$lib/stores";
  import { uuid } from "$lib/util";

  let editorHost: HTMLDivElement;
  let view: EditorView | null = null;
  let lastSavedContent = "";
  let savedAt = $state<Date | null>(null);

  let unsubscribeChapter: (() => void) | null = null;

  // Starter prompts shown when the chapter is empty
  const STARTER_PROMPTS = [
    "Begin with a sound — a door, a footstep, a held breath.",
    "Start with the weather. It's a cliché for a reason.",
    "Open on a hand. What is it holding? What is it letting go of?",
    "Begin with a question someone is afraid to answer.",
    "Start with the last thing they expected to see.",
    "Open on a memory — let the present intrude later.",
    "Begin mid-action. Catch the reader already running.",
    "Start with a smell. Memory lives there first.",
  ];

  let showStarter = $state(false);

  async function loadChapter(filename: string | null) {
    if (!view) return;
    const p = $project;
    if (!p || !filename) {
      view.setState(EditorState.create({ doc: "", extensions: extensions() }));
      lastSavedContent = "";
      showStarter = false;
      return;
    }
    try {
      const content = await api.chapterRead(p.root, filename);
      lastSavedContent = content;
      view.setState(
        EditorState.create({ doc: content, extensions: extensions() }),
      );
      showStarter = !content.trim();
    } catch (e) {
      notify("error", `Failed to load ${filename}: ${e}`);
    }
  }

  function extensions() {
    return [
      EditorView.lineWrapping,
      markdown(),
      syntaxHighlighting(defaultHighlightStyle),
      dracula,
      EditorView.theme({
        "&": { backgroundColor: "transparent", color: "var(--fg-0)" },
        ".cm-content": {
          fontFamily: "var(--serif)",
          fontSize: `${$fontSize}px`,
          lineHeight: "1.75",
          padding: "32px 48px 60vh 48px",
          caretColor: "var(--accent)",
          maxWidth: "720px",
          margin: "0 auto",
        },
        ".cm-gutters": { display: "none" },
        ".cm-activeLine": { backgroundColor: "rgba(0,0,0,0.02)" },
        ".cm-cursor": { borderLeftColor: "var(--accent)" },
        ".cm-selectionBackground, ::selection": {
          backgroundColor: "color-mix(in srgb, var(--accent-soft) 50%, transparent)",
        },
        "&.cm-focused": { outline: "none" },
      }),
      EditorView.updateListener.of((u: { docChanged: boolean }) => {
        if (u.docChanged) {
          scheduleSave();
          showStarter = false;
        }
      }),
    ];
  }

  // Recreate the editor when font size changes so the theme updates.
  let lastFontSize = $fontSize;
  $effect(() => {
    if ($fontSize !== lastFontSize && view) {
      lastFontSize = $fontSize;
      const content = view.state.doc.toString();
      view.destroy();
      view = new EditorView({
        state: EditorState.create({ doc: content, extensions: extensions() }),
        parent: editorHost,
      });
    }
  });

  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(saveCurrent, 1200);
  }

  async function saveCurrent() {
    const p = $project;
    const filename = $activeChapterFilename;
    if (!p || !filename || !view) return;
    const content = view.state.doc.toString();
    if (content === lastSavedContent) return;
    try {
      await api.chapterWrite(p.root, filename, content);
      lastSavedContent = content;
      savedAt = new Date();
      await refreshChapters();
      await refreshWritingStats();
    } catch (e) {
      notify("error", `Save failed: ${e}`);
    }
  }

  function applyStarterPrompt(prompt: string) {
    if (!view) return;
    const text = prompt + "\n\n";
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: text },
      selection: { anchor: text.length },
    });
    scheduleSave();
    showStarter = false;
  }

  export async function insertAtCursor(text: string) {
    if (!view) return;
    const sel = view.state.selection.main;
    view.dispatch({
      changes: { from: sel.from, to: sel.to, insert: text },
      selection: { anchor: sel.from + text.length },
    });
    scheduleSave();
  }

  export async function appendToDocument(text: string) {
    if (!view) return;
    const docLen = view.state.doc.length;
    const currentDoc = view.state.doc.toString();
    const needsPrefix = docLen > 0 && !currentDoc.endsWith("\n\n");
    const prefix = needsPrefix ? (currentDoc.endsWith("\n") ? "\n" : "\n\n") : "";
    const insertText = prefix + text;
    view.dispatch({
      changes: { from: docLen, to: docLen, insert: insertText },
      selection: { anchor: docLen + insertText.length },
    });
    scheduleSave();
  }

  export async function replaceSelection(text: string) {
    if (!view) return;
    const sel = view.state.selection.main;
    view.dispatch({
      changes: { from: sel.from, to: sel.to, insert: text },
      selection: { anchor: sel.from + text.length },
    });
    scheduleSave();
  }

  export function getSelection(): string {
    if (!view) return "";
    const sel = view.state.selection.main;
    return view.state.sliceDoc(sel.from, sel.to);
  }

  export function getPrefix(maxChars = 2000): string {
    if (!view) return "";
    const sel = view.state.selection.main;
    const from = Math.max(0, sel.from - maxChars);
    return view.state.sliceDoc(from, sel.from);
  }

  export function getContext(maxChars = 2000): string {
    if (!view) return "";
    const sel = view.state.selection.main;
    if (sel.from > 0) {
      const from = Math.max(0, sel.from - maxChars);
      return view.state.sliceDoc(from, sel.from);
    }
    const doc = view.state.doc.toString();
    if (doc.length <= maxChars) return doc;
    return doc.slice(0, maxChars);
  }

  export async function continueWriting() {
    if ($isGenerating) return;
    const prefix = getPrefix();
    if (prefix.trim().length < 10) {
      notify("info", "Write a few sentences first, then I can continue.");
      return;
    }
    const requestId = uuid();
    activeRequestId.set(requestId);
    isGenerating.set(true);

    const off = listenForChunks("ai_continue_chunk", requestId, (chunk) => {
      if (view && chunk.token) {
        const sel = view.state.selection.main;
        view.dispatch({
          changes: { from: sel.from, to: sel.to, insert: chunk.token },
          selection: { anchor: sel.from + chunk.token.length },
        });
      }
      if (chunk.finish) {
        isGenerating.set(false);
        activeRequestId.set(null);
        off();
        scheduleSave();
      }
    });

    try {
      await api.aiContinue(requestId, prefix, $genParams, $project?.root);
    } catch (e) {
      notify("error", `AI continue failed: ${e}`);
      isGenerating.set(false);
      activeRequestId.set(null);
      off();
    }
  }

  export async function rewriteSelection(instruction: string) {
    if ($isGenerating) return;
    const sel = getSelection();
    if (sel.trim().length < 5) {
      notify("info", "Select some text to rewrite first.");
      return;
    }
    const requestId = uuid();
    activeRequestId.set(requestId);
    isGenerating.set(true);

    let buf = "";
    const off = listenForChunks("ai_rewrite_chunk", requestId, (chunk) => {
      buf += chunk.token;
      if (chunk.finish) {
        if (view) {
          const s = view.state.selection.main;
          view.dispatch({
            changes: { from: s.from, to: s.to, insert: buf },
            selection: { anchor: s.from + buf.length },
          });
          scheduleSave();
        }
        isGenerating.set(false);
        activeRequestId.set(null);
        off();
      }
    });

    try {
      await api.aiRewrite(requestId, sel, instruction, $genParams, $project?.root);
    } catch (e) {
      notify("error", `AI rewrite failed: ${e}`);
      isGenerating.set(false);
      activeRequestId.set(null);
      off();
    }
  }

  function listenForChunks(
    event: string,
    requestId: string,
    cb: (chunk: { requestId: string; token: string; finish: boolean; error: string | null }) => void,
  ): () => void {
    let unlisten: (() => void) | null = null;
    let cancelled = false;
    (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<{
        requestId: string;
        token: string;
        finish: boolean;
        error: string | null;
      }>(event, (e) => {
        if (e.payload.requestId !== requestId) return;
        if (cancelled) return;
        cb(e.payload);
      });
    })();
    return () => {
      cancelled = true;
      if (unlisten) unlisten();
    };
  }

  onMount(async () => {
    view = new EditorView({
      state: EditorState.create({ doc: "", extensions: extensions() }),
      parent: editorHost,
    });
    unsubscribeChapter = activeChapterFilename.subscribe(loadChapter);
  });

  onDestroy(() => {
    if (saveTimer) clearTimeout(saveTimer);
    if (unsubscribeChapter) unsubscribeChapter();
    if (view) view.destroy();
  });

  let wordCount = $derived(view?.state.doc.toString().split(/\s+/).filter(Boolean).length ?? 0);
  let savedLabel = $derived(savedAt ? formatSaved(savedAt) : "");

  function formatSaved(date: Date): string {
    const diff = (Date.now() - date.getTime()) / 1000;
    if (diff < 5) return "just now";
    if (diff < 60) return `${Math.floor(diff)}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  }
</script>

<div class="editor-host-wrap" class:focus-mode={$focusMode}>
  <!-- Floating toolbar (chrome-faded in focus mode, reappears on hover) -->
  <div class="editor-toolbar chrome-fade">
    {#if $activeChapter}
      <div class="chapter-info">
        <span class="ch-title">{$activeChapter.title}</span>
        <span class="ch-meta">
          {wordCount} words
          {#if savedLabel}· saved {savedLabel}{/if}
        </span>
      </div>
    {/if}
    <div class="editor-tools">
      <button class="tool" onclick={() => setFontSize($fontSize - 1)} title="Smaller font">A−</button>
      <span class="font-size">{$fontSize}</span>
      <button class="tool" onclick={() => setFontSize($fontSize + 1)} title="Larger font">A+</button>
      <span class="divider"></span>
      <button class="tool" class:active={$typewriterMode} onclick={toggleTypewriter} title="Typewriter mode (keeps caret centered)">⌗</button>
      <button class="tool" class:active={$focusMode} onclick={toggleFocusMode} title="Focus mode — fades the chrome">
        {#if $focusMode}Exit{:else}Focus{/if}
      </button>
      <button class="tool" class:active={$aiPanelOpen} onclick={toggleAIPanel} title="AI Co-author">
        ✦ AI
      </button>
    </div>
  </div>

  <div class="editor-host" bind:this={editorHost}>
    {#if showStarter && $activeChapter}
      <div class="starter-overlay">
        <p class="starter-head">A blank page is permission. Start anywhere.</p>
        <div class="starter-prompts">
          {#each STARTER_PROMPTS.slice(0, 4) as prompt}
            <button class="starter-prompt" onclick={() => applyStarterPrompt(prompt)}>
              {prompt}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .editor-host-wrap {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--paper);
    position: relative;
  }
  .editor-host-wrap.focus-mode {
    background: var(--paper);
  }
  .editor-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 16px;
    background: color-mix(in srgb, var(--paper) 92%, transparent);
    border-bottom: 1px solid var(--border-soft);
    flex-shrink: 0;
    z-index: 5;
  }
  .focus-mode .editor-toolbar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    background: transparent;
    border-bottom: none;
  }
  .chapter-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .ch-title {
    font-family: var(--serif);
    font-size: 13px;
    font-weight: 500;
    color: var(--fg-0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ch-meta {
    font-size: 10px;
    color: var(--fg-2);
    font-variant-numeric: tabular-nums;
  }
  .editor-tools {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .tool {
    background: transparent;
    border: 1px solid var(--border-soft);
    border-radius: 4px;
    color: var(--fg-1);
    padding: 3px 8px;
    font-size: 11px;
    cursor: pointer;
    min-width: 26px;
  }
  .tool:hover {
    background: var(--bg-2);
    color: var(--fg-0);
    border-color: var(--accent-soft);
  }
  .tool.active {
    background: var(--accent-soft);
    color: var(--paper);
    border-color: var(--accent-soft);
  }
  .font-size {
    font-size: 10px;
    color: var(--fg-2);
    width: 18px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .divider {
    width: 1px;
    height: 16px;
    background: var(--border-soft);
    margin: 0 2px;
  }
  .editor-host {
    flex: 1;
    height: 100%;
    overflow: hidden;
    background: var(--paper);
    position: relative;
  }
  .editor-host :global(.cm-editor) {
    height: 100%;
  }
  .editor-host :global(.cm-scroller) {
    overflow: auto;
  }

  /* Starter prompts overlay */
  .starter-overlay {
    position: absolute;
    top: 60px;
    left: 50%;
    transform: translateX(-50%);
    width: 100%;
    max-width: 720px;
    padding: 0 48px;
    pointer-events: none;
    z-index: 3;
  }
  .starter-head {
    font-family: var(--serif);
    font-style: italic;
    font-size: 15px;
    color: var(--fg-2);
    margin: 0 0 16px 0;
  }
  .starter-prompts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    pointer-events: auto;
  }
  .starter-prompt {
    background: transparent;
    border: 1px solid var(--border-soft);
    border-radius: 14px;
    padding: 4px 12px;
    font-family: var(--serif);
    font-style: italic;
    font-size: 12px;
    color: var(--fg-1);
    cursor: pointer;
    pointer-events: auto;
  }
  .starter-prompt:hover {
    background: var(--bg-2);
    border-color: var(--accent-soft);
    color: var(--fg-0);
  }
</style>
