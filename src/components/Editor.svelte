<script lang="ts">
  import { onMount, onDestroy } from "svelte";
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
    genParams,
    isGenerating,
    notify,
    project,
    refreshChapters,
  } from "$lib/stores";
  import { uuid } from "$lib/util";

  let editorHost: HTMLDivElement;
  let view: EditorView | null = null;
  let lastSavedContent = "";

  let unsubscribeChapter: (() => void) | null = null;

  async function loadChapter(filename: string | null) {
    if (!view) return;
    const p = $project;
    if (!p || !filename) {
      view.setState(EditorState.create({ doc: "", extensions: extensions() }));
      lastSavedContent = "";
      return;
    }
    try {
      const content = await api.chapterRead(p.root, filename);
      lastSavedContent = content;
      view.setState(
        EditorState.create({ doc: content, extensions: extensions() }),
      );
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
          fontSize: "17px",
          lineHeight: "1.7",
          padding: "24px 32px",
          caretColor: "var(--accent)",
        },
        ".cm-gutters": { display: "none" },
        ".cm-activeLine": { backgroundColor: "rgba(255,255,255,0.02)" },
        ".cm-cursor": { borderLeftColor: "var(--accent)" },
        ".cm-selectionBackground, ::selection": {
          backgroundColor: "rgba(124,156,255,0.25)",
        },
      }),
      EditorView.updateListener.of((u: { docChanged: boolean }) => {
        if (u.docChanged) scheduleSave();
      }),
    ];
  }

  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  function scheduleSave() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(saveCurrent, 1500);
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
      await refreshChapters();
    } catch (e) {
      notify("error", `Save failed: ${e}`);
    }
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

    let buf = "";
    const off = listenForChunks("ai_continue_chunk", requestId, (chunk) => {
      buf += chunk.token;
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
      await api.aiContinue(requestId, prefix, $genParams);
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
      await api.aiRewrite(requestId, sel, instruction, $genParams);
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
</script>

<div class="editor-host" bind:this={editorHost}></div>

<style>
  .editor-host {
    flex: 1;
    height: 100%;
    overflow: hidden;
    background: var(--bg-1);
  }
  .editor-host :global(.cm-editor) {
    height: 100%;
  }
  .editor-host :global(.cm-scroller) {
    overflow: auto;
  }
</style>
