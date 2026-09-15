<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$components/Sidebar.svelte";
  import Toolbar from "$components/Toolbar.svelte";
  import Editor from "$components/Editor.svelte";
  import Terminal from "$components/Terminal.svelte";
  import AIPanel from "$components/AIPanel.svelte";
  import StatusBar from "$components/StatusBar.svelte";
  import { refreshModelStatus, refreshWorkspaceBooks, workspaceBooks, project, openProjectFolder } from "$lib/stores";
  import { get } from "svelte/store";

  let editor = $state<Editor>();
  let aiPanelWidth = $state(340);
  let terminalHeight = $state(220);
  let showTerminal = $state(true);

  onMount(async () => {
    await refreshModelStatus();
    await refreshWorkspaceBooks();
    const books = get(workspaceBooks);
    if (!get(project) && books.length > 0) {
      await openProjectFolder(books[0].root);
    }
  });

  function startContinue() {
    editor?.continueWriting();
  }
  function startRewrite(instruction: string) {
    editor?.rewriteSelection(instruction);
  }

  function startResizeTerminal(e: MouseEvent) {
    e.preventDefault();
    const startY = e.clientY;
    const startH = terminalHeight;
    const move = (ev: MouseEvent) => {
      const dy = startY - ev.clientY;
      terminalHeight = Math.max(80, Math.min(600, startH + dy));
    };
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }

  function startResizeAI(e: MouseEvent) {
    e.preventDefault();
    const startX = e.clientX;
    const startW = aiPanelWidth;
    const move = (ev: MouseEvent) => {
      const dx = startX - ev.clientX;
      aiPanelWidth = Math.max(260, Math.min(600, startW + dx));
    };
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  }
</script>

<div class="app-shell">
  <Sidebar />

  <div class="center">
    <Toolbar onContinue={startContinue} onRewrite={startRewrite} />

    <div class="editor-wrap">
      <Editor bind:this={editor} />
    </div>

    {#if showTerminal}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div class="terminal-resizer" role="separator" aria-orientation="horizontal" onmousedown={startResizeTerminal}></div>
      <div class="terminal-wrap" style="height: {terminalHeight}px">
        <Terminal />
      </div>
    {/if}

    <div class="terminal-toggle">
      <button onclick={() => (showTerminal = !showTerminal)}>
        {showTerminal ? "Hide terminal" : "Show terminal"}
      </button>
    </div>
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="ai-resizer" role="separator" aria-orientation="vertical" onmousedown={startResizeAI}></div>
  <div class="ai-wrap" style="width: {aiPanelWidth}px">
    <AIPanel />
  </div>
</div>

<StatusBar />

<style>
  .app-shell {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }
  .center {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }
  .editor-wrap {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
  }
  .terminal-resizer {
    height: 4px;
    background: var(--border);
    cursor: row-resize;
    flex-shrink: 0;
  }
  .terminal-resizer:hover {
    background: var(--accent);
  }
  .terminal-wrap {
    flex-shrink: 0;
    overflow: hidden;
  }
  .ai-resizer {
    width: 4px;
    background: var(--border);
    cursor: col-resize;
    flex-shrink: 0;
  }
  .ai-resizer:hover {
    background: var(--accent);
  }
  .ai-wrap {
    flex-shrink: 0;
    overflow: hidden;
  }
  .terminal-toggle {
    height: 24px;
    background: var(--bg-2);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 8px;
    flex-shrink: 0;
  }
  .terminal-toggle button {
    font-size: 10px;
    padding: 2px 8px;
  }
</style>
