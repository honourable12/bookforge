<script lang="ts">
  import { get } from "svelte/store";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    activeChapterFilename,
    createBook,
    notify,
    openProjectFolder,
    project,
    refreshChapters,
    refreshWritingStats,
    workspaceBooks,
    workspacePath,
    view,
    writingStats,
  } from "$lib/stores";
  import { api } from "$lib/api";
  import { humanDate, formatNumber, formatRelative } from "$lib/util";
  import ProgressRing from "./ProgressRing.svelte";

  let newBookTitle = $state("");
  let newBookAuthor = $state("");
  let showNewBook = $state(false);
  let isCreatingBook = $state(false);
  let showLibrary = $state(false);

  let newChapterName = $state("");
  let newChapterTitle = $state("");
  let showNewChapter = $state(false);

  async function handleCreateBook() {
    const title = newBookTitle.trim();
    if (!title) return;
    isCreatingBook = true;
    try {
      await createBook(title, newBookAuthor.trim() || undefined);
      newBookTitle = "";
      newBookAuthor = "";
      showNewBook = false;
    } catch {
      // handled by createBook notify
    } finally {
      isCreatingBook = false;
    }
  }

  async function pickAndOpen() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Open BookForge project folder",
    });
    if (typeof selected === "string") {
      await openProjectFolder(selected);
    }
  }

  async function selectChapter(filename: string) {
    activeChapterFilename.set(filename);
    view.set("writer");
  }

  async function addChapter() {
    const p = get(project);
    if (!p) return;
    const name = newChapterName.trim().endsWith(".md")
      ? newChapterName.trim()
      : `${newChapterName.trim() || "chapter"}.md`;
    try {
      await api.chapterCreate(p.root, name, newChapterTitle || name.replace(/\.md$/, ""));
      newChapterName = "";
      newChapterTitle = "";
      showNewChapter = false;
      await refreshChapters();
      activeChapterFilename.set(name);
    } catch (e) {
      notify("error", `Create chapter: ${e}`);
    }
  }

  async function deleteChapter(filename: string) {
    const p = get(project);
    if (!p) return;
    if (!confirm(`Delete ${filename}? This cannot be undone.`)) return;
    try {
      await api.chapterDelete(p.root, filename);
      if (get(activeChapterFilename) === filename) {
        activeChapterFilename.set(null);
      }
      await refreshChapters();
      await refreshWritingStats();
      notify("success", `Deleted ${filename}`);
    } catch (e) {
      notify("error", `Delete failed: ${e}`);
    }
  }

  let totalWords = $derived(
    $project ? $project.chapters.reduce((s, c) => s + c.wordCount, 0) : 0,
  );

  let todayWords = $derived($writingStats?.todayWords ?? 0);
  let dailyGoal = $derived($writingStats?.dailyGoal ?? 500);
  let goalMet = $derived($writingStats?.todayGoalMet ?? false);
  let streak = $derived($writingStats?.currentStreak ?? 0);

  let navItems = $derived([
    { id: "writer" as const, label: "Write", icon: "✎" },
    { id: "bible" as const, label: "Story Bible", icon: "📖" },
    { id: "map" as const, label: "Story Map", icon: "🗺" },
    { id: "stats" as const, label: "Progress", icon: "📊" },
  ]);
</script>

<aside class="sidebar chrome-fade">
  <!-- Top: brand + library nav -->
  <div class="sidebar-section brand-row">
    <button class="brand" onclick={() => view.set("library")}>
      <span class="brand-icon">✎</span>
      <span class="brand-text">BookForge</span>
    </button>
  </div>

  {#if $project}
    <!-- Book info -->
    <div class="sidebar-section book-info">
      <h2 class="book-title">{$project.meta.title}</h2>
      <div class="book-meta">by {$project.meta.author}</div>
      <div class="book-stats">
        <span>{formatNumber(totalWords)} words</span>
        {#if streak > 0}
          <span class="streak-badge">
            🔥 <span class="flame">{streak}</span>
          </span>
        {/if}
      </div>
      <!-- Daily goal ring -->
      <div class="goal-row">
        <ProgressRing value={todayWords} max={dailyGoal} size={40} stroke={3} showLabel={false} />
        <div class="goal-text">
          <div class="goal-numbers">
            <span class="today-words" class:met={goalMet}>{formatNumber(todayWords)}</span>
            <span class="goal-target"> / {formatNumber(dailyGoal)}</span>
          </div>
          <div class="goal-label">
            {#if goalMet}
              ✓ Goal met today
            {:else}
              {formatNumber(Math.max(0, dailyGoal - todayWords))} to go today
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Nav -->
    <div class="sidebar-section nav-section">
      {#each navItems as item}
        <button
          class="nav-item"
          class:active={$view === item.id}
          onclick={() => view.set(item.id)}
        >
          <span class="nav-icon">{item.icon}</span>
          <span>{item.label}</span>
        </button>
      {/each}
    </div>

    <!-- Chapters -->
    <div class="sidebar-section flex-grow">
      <div class="section-header">
        <span>Chapters</span>
        <button class="action-btn" onclick={() => (showNewChapter = !showNewChapter)}>+ Add</button>
      </div>

      {#if showNewChapter}
        <div class="new-chapter">
          <input bind:value={newChapterName} placeholder="filename.md" />
          <input bind:value={newChapterTitle} placeholder="Chapter title" />
          <div class="row">
            <button class="primary" onclick={addChapter}>Create</button>
            <button onclick={() => (showNewChapter = false)}>Cancel</button>
          </div>
        </div>
      {/if}

      {#if $project && $project.chapters.length > 0}
        <ul class="chapter-list">
          {#each $project.chapters as ch (ch.filename)}
            <li>
              <button
                class="chapter-item"
                class:active={$activeChapterFilename === ch.filename}
                onclick={() => selectChapter(ch.filename)}
              >
                <span class="chapter-order">{String(ch.order).padStart(2, "0")}</span>
                <span class="chapter-title" title={ch.filename}>{ch.title}</span>
                <span class="chapter-words">{ch.wordCount}w</span>
              </button>
              <button class="chapter-delete" onclick={() => deleteChapter(ch.filename)} title="Delete">×</button>
            </li>
          {/each}
        </ul>
      {:else}
        <div class="empty">
          <p>No chapters yet.</p>
        </div>
      {/if}
    </div>
  {:else}
    <!-- No project open -->
    <div class="sidebar-section flex-grow">
      <div class="empty">
        <p>No book open.</p>
        <button class="primary empty-create-btn" onclick={() => (showNewBook = true)}>+ Create New Book</button>
        <div class="empty-hint">Books are stored in <code>{$workspacePath || "~/.bookforge"}</code></div>
      </div>
    </div>
  {/if}

  <!-- Library / new book actions -->
  <div class="sidebar-section actions-section">
    <div class="row">
      <button class="action-btn full" onclick={() => (showNewBook = !showNewBook)}>+ New Book</button>
      <button class="action-btn full" onclick={() => (showLibrary = !showLibrary)}>Books ({$workspaceBooks.length})</button>
      <button class="action-btn full" onclick={pickAndOpen}>Open…</button>
    </div>
  </div>

  {#if showNewBook}
    <div class="new-book-form">
      <div class="form-heading">Create Book</div>
      <input
        bind:value={newBookTitle}
        placeholder="Book Name"
        onkeydown={(e) => e.key === "Enter" && handleCreateBook()}
      />
      <input
        bind:value={newBookAuthor}
        placeholder="Author (optional)"
        onkeydown={(e) => e.key === "Enter" && handleCreateBook()}
      />
      <div class="workspace-hint">
        Created in <code>{$workspacePath || "~/.bookforge"}</code>
      </div>
      <div class="row">
        <button class="primary" onclick={handleCreateBook} disabled={!newBookTitle.trim() || isCreatingBook}>
          {isCreatingBook ? "Creating…" : "Create"}
        </button>
        <button onclick={() => (showNewBook = false)}>Cancel</button>
      </div>
    </div>
  {/if}

  {#if showLibrary}
    <div class="library-browser">
      <div class="library-header">
        <span>Library ({$workspaceBooks.length})</span>
        <button onclick={() => (showLibrary = false)}>✕</button>
      </div>
      <ul class="library-list">
        {#each $workspaceBooks as b (b.root)}
          <li>
            <button
              class="library-item"
              class:active={$project?.root === b.root}
              onclick={() => {
                openProjectFolder(b.root);
                showLibrary = false;
              }}
            >
              <div class="lib-title">{b.meta.title}</div>
              <div class="lib-meta">{b.chapters.length} ch · {formatRelative(b.meta.updatedAt)}</div>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</aside>

<style>
  .sidebar {
    width: 240px;
    min-width: 240px;
    background: var(--bg-1);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .sidebar-section {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-soft);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .flex-grow {
    flex: 1;
    overflow-y: auto;
    border-bottom: none;
  }
  .brand-row {
    padding: 12px 12px 10px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: none;
    color: var(--fg-0);
    cursor: pointer;
    padding: 0;
    font-size: 14px;
  }
  .brand:hover {
    background: transparent;
    color: var(--fg-0);
  }
  .brand-icon {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--paper);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--serif);
    font-size: 15px;
  }
  .brand-text {
    font-family: var(--serif);
    font-weight: 500;
    letter-spacing: -0.01em;
  }
  .book-info {
    padding: 12px;
  }
  .book-title {
    margin: 0;
    font-family: var(--serif);
    font-size: 15px;
    font-weight: 500;
    color: var(--fg-0);
    line-height: 1.25;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .book-meta {
    font-size: 11px;
    color: var(--fg-2);
    margin-top: 2px;
  }
  .book-stats {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
    font-size: 11px;
    color: var(--fg-1);
    font-variant-numeric: tabular-nums;
  }
  .streak-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--bg-3);
    color: var(--warn);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
  }
  .goal-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 10px;
    padding: 8px;
    background: var(--bg-2);
    border-radius: 8px;
  }
  .goal-text {
    flex: 1;
    min-width: 0;
  }
  .goal-numbers {
    display: flex;
    align-items: baseline;
    gap: 2px;
    font-size: 12px;
  }
  .today-words {
    font-family: var(--serif);
    font-weight: 500;
    color: var(--fg-0);
    font-variant-numeric: tabular-nums;
  }
  .today-words.met {
    color: var(--success);
  }
  .goal-target {
    color: var(--fg-2);
    font-size: 11px;
  }
  .goal-label {
    font-size: 10px;
    color: var(--fg-2);
    margin-top: 2px;
  }
  .nav-section {
    padding: 6px 8px;
    gap: 1px;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    background: transparent;
    border: none;
    color: var(--fg-1);
    font-size: 12px;
    font-weight: 500;
    padding: 6px 8px;
    border-radius: 5px;
    cursor: pointer;
    text-align: left;
  }
  .nav-item:hover {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .nav-item.active {
    background: var(--bg-3);
    color: var(--accent);
  }
  .nav-icon {
    font-size: 14px;
    width: 16px;
    text-align: center;
  }
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-2);
    font-weight: 600;
    margin-bottom: 6px;
  }
  .action-btn {
    font-size: 11px;
    padding: 4px 8px;
  }
  .full {
    width: 100%;
  }
  .row {
    display: flex;
    gap: 4px;
    flex-direction: column;
  }
  .actions-section {
    gap: 4px;
  }
  .new-book-form,
  .library-browser,
  .new-chapter {
    margin: 6px 12px;
    padding: 10px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .new-book-form .form-heading {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--accent);
    margin-bottom: 6px;
  }
  .new-book-form input,
  .new-chapter input {
    width: 100%;
    font-size: 12px;
    margin-bottom: 6px;
  }
  .new-book-form .row,
  .new-chapter .row {
    flex-direction: row;
  }
  .new-book-form button,
  .new-chapter button {
    flex: 1;
    padding: 4px 8px;
    font-size: 11px;
  }
  .workspace-hint {
    font-size: 10px;
    color: var(--fg-2);
    line-height: 1.4;
    margin-bottom: 6px;
  }
  .workspace-hint code {
    font-family: var(--mono);
    background: var(--bg-1);
    padding: 1px 4px;
    border-radius: 3px;
    word-break: break-all;
  }
  .library-browser {
    max-height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    font-weight: 600;
    color: var(--fg-2);
    text-transform: uppercase;
  }
  .library-header button {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 11px;
    color: var(--fg-2);
    padding: 0 4px;
  }
  .library-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .library-item {
    width: 100%;
    text-align: left;
    padding: 5px 8px;
    background: var(--bg-1);
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .library-item:hover {
    border-color: var(--border);
    background: var(--bg-3);
  }
  .library-item.active {
    border-color: var(--accent);
  }
  .lib-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--fg-0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .lib-meta {
    font-size: 10px;
    color: var(--fg-2);
  }
  .empty {
    color: var(--fg-2);
    font-size: 12px;
    text-align: center;
    padding: 24px 8px;
    line-height: 1.5;
  }
  .empty code {
    background: var(--bg-2);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .empty-create-btn {
    margin-top: 8px;
    width: 100%;
  }
  .empty-hint {
    margin-top: 8px;
    font-size: 10px;
    color: var(--fg-2);
    line-height: 1.4;
  }
  .empty-hint code {
    font-family: var(--mono);
    word-break: break-all;
  }
  .new-chapter input {
    font-size: 12px;
  }
  .chapter-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .chapter-list li {
    display: flex;
    align-items: stretch;
  }
  .chapter-item {
    flex: 1;
    display: grid;
    grid-template-columns: 28px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    text-align: left;
    font-size: 12px;
    color: var(--fg-1);
    cursor: pointer;
  }
  .chapter-item:hover {
    background: var(--bg-2);
  }
  .chapter-item.active {
    background: var(--bg-3);
    color: var(--fg-0);
    border-color: var(--border);
  }
  .chapter-order {
    color: var(--fg-2);
    font-family: var(--mono);
    font-size: 10px;
  }
  .chapter-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--serif);
  }
  .chapter-words {
    color: var(--fg-2);
    font-size: 10px;
    font-family: var(--mono);
  }
  .chapter-delete {
    background: transparent;
    border: none;
    color: var(--fg-2);
    padding: 0 6px;
    border-radius: 4px;
    cursor: pointer;
    opacity: 0;
    font-size: 14px;
  }
  .chapter-list li:hover .chapter-delete {
    opacity: 1;
  }
  .chapter-delete:hover {
    color: var(--error);
    background: rgba(248, 113, 113, 0.1);
  }
</style>
