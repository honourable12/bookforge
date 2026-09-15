<script lang="ts">
  import { get } from "svelte/store";
  import { api } from "$lib/api";
  import {
    activeChapter,
    busy,
    notify,
    project,
    refreshChapters,
  } from "$lib/stores";
  import { formatBytes } from "$lib/util";
  import type { ExportResult } from "$lib/types";

  interface Props {
    onContinue: () => void;
    onRewrite: (instruction: string) => void;
  }

  let { onContinue, onRewrite }: Props = $props();

  let rewriteInstruction = $state("");
  let showRewrite = $state(false);

  async function exportTo(format: "html" | "pdf" | "epub") {
    const p = get(project);
    if (!p) {
      notify("error", "Open a project first.");
      return;
    }
    busy.set(true);
    try {
      let r: ExportResult;
      if (format === "html") r = await api.exportHtml(p.root);
      else if (format === "pdf") r = await api.exportPdf(p.root);
      else r = await api.exportEpub(p.root);
      notify("success", `Exported ${format.toUpperCase()} (${formatBytes(r.sizeBytes)}) → ${r.path}`);
    } catch (e) {
      notify("error", `Export failed: ${e}`);
    } finally {
      busy.set(false);
    }
  }

  async function newChapterQuick() {
    const p = get(project);
    if (!p) return;
    const idx = p.chapters.length + 1;
    const name = `${String(idx).padStart(2, "0")}-chapter.md`;
    try {
      await api.chapterCreate(p.root, name, `Chapter ${idx}`);
      await refreshChapters();
      notify("success", `Added ${name}`);
    } catch (e) {
      notify("error", `${e}`);
    }
  }

  function triggerContinue() {
    onContinue();
  }

  function triggerRewrite() {
    if (!get(activeChapter)) {
      notify("info", "Open a chapter first.");
      return;
    }
    showRewrite = !showRewrite;
  }

  export async function doRewrite() {
    onRewrite(rewriteInstruction);
    rewriteInstruction = "";
    showRewrite = false;
  }
</script>

<div class="toolbar">
  <div class="group">
    <button onclick={newChapterQuick} disabled={!$project}>+ Chapter</button>
  </div>

  <div class="group">
    <button onclick={triggerContinue} disabled={!$activeChapter}>✦ Continue</button>
    <button onclick={triggerRewrite} disabled={!$activeChapter}>✎ Rewrite</button>
  </div>

  {#if showRewrite}
    <div class="rewrite-inline">
      <input
        bind:value={rewriteInstruction}
        placeholder="e.g. make it more lyrical, switch to past tense"
        onkeydown={(e) => e.key === "Enter" && doRewrite()}
      />
      <button class="primary" onclick={doRewrite}>Apply</button>
      <button onclick={() => (showRewrite = false)}>×</button>
    </div>
  {/if}

  <div class="spacer"></div>

  <div class="group">
    <span class="label">Export:</span>
    <button onclick={() => exportTo("html")} disabled={!$project || $busy}>HTML</button>
    <button onclick={() => exportTo("pdf")} disabled={!$project || $busy}>PDF</button>
    <button onclick={() => exportTo("epub")} disabled={!$project || $busy}>EPUB</button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }
  .group {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .group .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-2);
    margin-right: 4px;
  }
  .toolbar button {
    padding: 4px 10px;
    font-size: 12px;
  }
  .spacer {
    flex: 1;
  }
  .rewrite-inline {
    display: flex;
    gap: 4px;
    align-items: center;
    background: var(--bg-1);
    padding: 4px;
    border-radius: 6px;
    border: 1px solid var(--border);
  }
  .rewrite-inline input {
    font-size: 12px;
    width: 280px;
  }
  .rewrite-inline button {
    padding: 3px 8px;
    font-size: 11px;
  }
</style>
