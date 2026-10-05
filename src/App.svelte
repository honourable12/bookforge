<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import Sidebar from "$components/Sidebar.svelte";
  import Toolbar from "$components/Toolbar.svelte";
  import Editor from "$components/Editor.svelte";
  import Terminal from "$components/Terminal.svelte";
  import AIPanel from "$components/AIPanel.svelte";
  import StatusBar from "$components/StatusBar.svelte";
  import LibraryView from "$components/LibraryView.svelte";
  import StoryBible from "$components/StoryBible.svelte";
  import StatsView from "$components/StatsView.svelte";
  import ThemeSwitcher from "$components/ThemeSwitcher.svelte";
  import {
    refreshModelStatus,
    refreshWorkspaceBooks,
    workspaceBooks,
    project,
    openProjectFolder,
    view,
    focusMode,
    aiPanelOpen,
    showTerminal,
    loadPrefs,
    toggleFocusMode,
    toggleSidebar,
  } from "$lib/stores";

  let editor = $state<Editor>();
  let aiPanelWidth = $state(340);
  let terminalHeight = $state(220);

  onMount(async () => {
    // Each Tauri call is wrapped in its own try/catch so a single
    // failure (e.g. prefs_load if the Rust backend wasn't rebuilt with
    // the new commands) doesn't kill the rest of the mount and leave
    // a blank screen.
    try {
      await loadPrefs();
    } catch (e) {
      console.warn("loadPrefs failed:", e);
    }
    try {
      await refreshModelStatus();
    } catch (e) {
      console.warn("refreshModelStatus failed:", e);
    }
    try {
      await refreshWorkspaceBooks();
      const books = get(workspaceBooks);
      if (!get(project) && books.length > 0) {
        await openProjectFolder(books[0].root);
      }
    } catch (e) {
      console.warn("workspace load failed:", e);
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

  // Global keyboard shortcuts
  function onKeydown(e: KeyboardEvent) {
    // ESC exits focus mode
    if (e.key === "Escape" && get(focusMode)) {
      e.preventDefault();
      toggleFocusMode();
      return;
    }
    // Cmd/Ctrl + . toggles focus mode
    if ((e.metaKey || e.ctrlKey) && e.key === ".") {
      e.preventDefault();
      toggleFocusMode();
      return;
    }
    // Cmd/Ctrl + B toggles sidebar (not in focus mode)
    if ((e.metaKey || e.ctrlKey) && e.key === "b" && !get(focusMode)) {
      e.preventDefault();
      toggleSidebar();
      return;
    }
  }

  onMount(() => {
    window.addEventListener("keydown", onKeydown);
  });
  onDestroy(() => {
    window.removeEventListener("keydown", onKeydown);
  });

  let inWriterView = $derived($view === "writer" && !!$project);
  let inLibraryView = $derived($view === "library" || !$project);
</script>

{#if $inLibraryView}
  <LibraryView />
  <StatusBar />
{:else}
  <div class="app-shell" class:focus-mode={$focusMode}>
    {#if !$focusMode}
      <Sidebar />
    {/if}

    <div class="center">
      {#if !$focusMode}
        <div class="top-strip chrome-fade">
          <nav class="view-nav">
            <button class="nav-btn" class:active={$view === "writer"} onclick={() => view.set("writer")}>Write</button>
            <button class="nav-btn" class:active={$view === "bible"} onclick={() => view.set("bible")}>Story Bible</button>
            <button class="nav-btn" class:active={$view === "stats"} onclick={() => view.set("stats")}>Progress</button>
          </nav>
          <div class="top-right">
            <ThemeSwitcher />
          </div>
        </div>
        {#if $view === "writer"}
          <Toolbar onContinue={startContinue} onRewrite={startRewrite} />
        {/if}
      {/if}

      {#if $view === "writer"}
        <div class="editor-wrap">
          <Editor bind:this={editor} />
        </div>

        {#if $showTerminal && !$focusMode}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div class="terminal-resizer" role="separator" aria-orientation="horizontal" onmousedown={startResizeTerminal}></div>
          <div class="terminal-wrap" style="height: {terminalHeight}px">
            <Terminal />
          </div>
        {/if}
      {:else if $view === "bible"}
        <StoryBible />
      {:else if $view === "stats"}
        <StatsView />
      {/if}
    </div>

    {#if $view === "writer" && $aiPanelOpen && !$focusMode}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div class="ai-resizer" role="separator" aria-orientation="vertical" onmousedown={startResizeAI}></div>
      <div class="ai-wrap" style="width: {aiPanelWidth}px">
        <AIPanel
          onInsertText={(text) => editor?.insertAtCursor(text)}
          onAppendText={(text) => editor?.appendToDocument(text)}
          onReplaceText={(text) => editor?.replaceSelection(text)}
          getContext={() => editor?.getContext() || ""}
        />
      </div>
    {/if}
  </div>

  <StatusBar />

  {#if $focusMode}
    <div class="focus-hint">Press ESC to exit focus mode</div>
  {/if}
{/if}

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
  .top-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border-soft);
    flex-shrink: 0;
  }
  .view-nav {
    display: flex;
    gap: 2px;
  }
  .nav-btn {
    background: transparent;
    border: 1px solid transparent;
    color: var(--fg-1);
    font-size: 12px;
    font-weight: 500;
    padding: 4px 12px;
    border-radius: 5px;
    cursor: pointer;
  }
  .nav-btn:hover {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .nav-btn.active {
    background: var(--bg-3);
    color: var(--accent);
    border-color: var(--border-soft);
  }
  .top-right {
    display: flex;
    align-items: center;
    gap: 6px;
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
  .focus-hint {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10;
    font-size: 11px;
    color: var(--fg-2);
    pointer-events: none;
    opacity: 0.7;
    animation: fadeIn 0.6s 0.4s ease both;
  }
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 0.7; }
  }
</style>
