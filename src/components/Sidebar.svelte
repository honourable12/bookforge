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
    workspaceBooks,
    workspacePath,
  } from "$lib/stores";
  import { api } from "$lib/api";
  import { humanDate } from "$lib/util";

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
    } catch (e) {
      // error handled by createBook notify
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
      notify("success", `Deleted ${filename}`);
    } catch (e) {
      notify("error", `Delete failed: ${e}`);
    }
  }
</script>

<aside class="sidebar">
  <div class="sidebar-section">
    <div class="section-header">
      <span>Book</span>
      <div class="actions">
        <button class="action-btn" onclick={() => (showNewBook = !showNewBook)} title="Create a new book in ~/.bookforge">
          + New
        </button>
        {#if $workspaceBooks.length > 0}
          <button class="action-btn" onclick={() => (showLibrary = !showLibrary)} title="Browse library in ~/.bookforge">
            Books ({$workspaceBooks.length})
          </button>
        {/if}
        <button class="action-btn" onclick={pickAndOpen} title="Open an external folder">
          Open…
        </button>
      </div>
    </div>

    {#if showNewBook}
      <div class="new-book-form">
        <div class="form-heading">Create Book</div>
        <input
          bind:value={newBookTitle}
          placeholder="Book Name (e.g. Chronicles of Earth)"
          onkeydown={(e) => e.key === "Enter" && handleCreateBook()}
        />
        <input
          bind:value={newBookAuthor}
          placeholder="Author name (optional)"
          onkeydown={(e) => e.key === "Enter" && handleCreateBook()}
        />
        <div class="workspace-hint">
          📁 Automatically created in <code>{$workspacePath || ".bookforge"}</code>
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
                <div class="lib-meta">{b.chapters.length} ch · {humanDate(b.meta.updatedAt)}</div>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    {#if $project}
      <div class="project-info">
        <div class="project-title" title={$project.meta.title}>{$project.meta.title}</div>
        <div class="project-meta">by {$project.meta.author}</div>
        <div class="project-meta">{$project.chapters.length} chapters · v{$project.meta.version}</div>
        <div class="project-meta" title={$project.root}>{$project.root}</div>
        <div class="project-meta">Updated {humanDate($project.meta.updatedAt)}</div>
      </div>
    {:else if !showNewBook}
      <div class="empty">
        <p>No book open.</p>
        <button class="primary empty-create-btn" onclick={() => (showNewBook = true)}>+ Create New Book</button>
        <div class="empty-hint">Books are stored in <code>{$workspacePath || "C:\\Users\\user\\.bookforge"}</code></div>
      </div>
    {/if}
  </div>

  <div class="sidebar-section flex-grow">
    <div class="section-header">
      <span>Chapters</span>
      {#if $project}
        <button onclick={() => (showNewChapter = !showNewChapter)}>+ Add</button>
      {/if}
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
</aside>

<style>
  .sidebar {
    width: 280px;
    min-width: 280px;
    background: var(--bg-1);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .sidebar-section {
    padding: 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .flex-grow {
    flex: 1;
    overflow-y: auto;
    border-bottom: none;
  }
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-2);
    font-weight: 600;
  }
  .section-header .actions {
    display: flex;
    gap: 4px;
  }
  .section-header button {
    padding: 2px 8px;
    font-size: 11px;
  }
  .project-info {
    font-size: 12px;
    line-height: 1.5;
  }
  .project-title {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .project-meta {
    color: var(--fg-2);
    font-size: 11px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .empty {
    color: var(--fg-2);
    font-size: 12px;
    text-align: center;
    padding: 16px 8px;
    line-height: 1.5;
  }
  .empty code {
    background: var(--bg-2);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .new-book-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    margin-bottom: 6px;
  }
  .new-book-form .form-heading {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--accent);
  }
  .new-book-form input {
    font-size: 12px;
  }
  .new-book-form .row {
    display: flex;
    gap: 6px;
  }
  .new-book-form button {
    flex: 1;
    padding: 4px 8px;
    font-size: 11px;
  }
  .workspace-hint {
    font-size: 10px;
    color: var(--fg-2);
    line-height: 1.4;
  }
  .workspace-hint code {
    font-family: var(--mono);
    background: var(--bg-1);
    padding: 1px 4px;
    border-radius: 3px;
    word-break: break-all;
  }
  .library-browser {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px;
    margin-bottom: 6px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 180px;
    overflow-y: auto;
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
    background: var(--bg-3);
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
    font-family: var(--mono);
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
  .new-chapter {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    background: var(--bg-2);
    border-radius: 6px;
    margin-bottom: 4px;
  }
  .new-chapter input {
    font-size: 12px;
  }
  .new-chapter .row {
    display: flex;
    gap: 6px;
  }
  .new-chapter button {
    flex: 1;
    padding: 4px 8px;
    font-size: 11px;
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
    gap: 0;
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
