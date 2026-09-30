<script lang="ts">
  import { onMount } from "svelte";
  import {
    workspaceBooks,
    openProjectFolder,
    refreshWorkspaceBooks,
  } from "$lib/stores";
  import { formatRelative, formatNumber } from "$lib/util";

  let showCreate = $state(false);
  let creating = $state(false);
  let title = $state("");
  let author = $state("");
  let dailyGoal = $state(500);

  onMount(async () => {
    await refreshWorkspaceBooks();
  });

  function timeOfDay() {
    const h = new Date().getHours();
    if (h < 5) return "Late night";
    if (h < 12) return "Good morning";
    if (h < 17) return "Good afternoon";
    if (h < 21) return "Good evening";
    return "Late night";
  }

  let totalWordsToday = $derived(
    $workspaceBooks.reduce((s, b) => {
      const today = new Date().toISOString().split("T")[0];
      return s + (b.chapters.length > 0 ? 0 : 0);
    }, 0)
  );

  let activeStreak = $derived(
    $workspaceBooks.reduce((max, b) => Math.max(max, 0), 0)
  );

  async function handleCreate() {
    if (!title.trim()) return;
    creating = true;
    try {
      const { createBook } = await import("$lib/stores");
      await createBook(title.trim(), author.trim() || undefined);
      title = "";
      author = "";
      dailyGoal = 500;
      showCreate = false;
    } catch {
      // handled by createBook notify
    } finally {
      creating = false;
    }
  }
</script>

<div class="library-view">
  <header class="header">
    <div class="brand">
      <div class="logo">✎</div>
      <div>
        <h1 class="title">BookForge</h1>
        <p class="tagline">Where stories take shape</p>
      </div>
    </div>
    <button class="primary new-btn" onclick={() => (showCreate = !showCreate)}>
      + New Book
    </button>
  </header>

  <main class="main">
    <section class="greeting">
      <h2 class="greet">{timeOfDay()}.</h2>
      <p class="sub">
        {#if $workspaceBooks.length === 0}
          Your library is empty. Time to begin the first page.
        {:else if totalWordsToday > 0}
          You've written {formatNumber(totalWordsToday)} words today — keep going.
        {:else if activeStreak > 0}
          Welcome back. Your {activeStreak}-day streak is waiting — write a sentence to keep it alive.
        {:else}
          A blank page is the only honest beginning.
        {/if}
      </p>
    </section>

    {#if showCreate}
      <div class="create-card fade-in">
        <h3>What's your book about?</h3>
        <p class="hint">A title to begin. You can change everything later — even the title.</p>
        <input bind:value={title} placeholder="The Salt Path Home" onkeydown={(e) => e.key === "Enter" && handleCreate()} />
        <input bind:value={author} placeholder="Author (optional)" onkeydown={(e) => e.key === "Enter" && handleCreate()} />
        <div class="goal-row">
          <label for="goal">Daily goal</label>
          <input id="goal" type="number" bind:value={dailyGoal} min="50" max="2000" step="50" />
          <span class="goal-hint">words / day</span>
        </div>
        <div class="actions">
          <button onclick={() => (showCreate = false)}>Cancel</button>
          <button class="primary" onclick={handleCreate} disabled={!title.trim() || creating}>
            {creating ? "Creating…" : "Begin writing"}
          </button>
        </div>
      </div>
    {/if}

    {#if $workspaceBooks.length === 0 && !showCreate}
      <div class="empty">
        <div class="empty-glow"></div>
        <div class="empty-content">
          <div class="empty-icon">✦</div>
          <h3>Every great book begins blank.</h3>
          <p>Create your first book. Set a small daily goal.<br />Write one sentence today. That's how every library begins.</p>
          <button class="primary big" onclick={() => (showCreate = true)}>Begin your first book</button>
        </div>
      </div>
    {:else if $workspaceBooks.length > 0}
      <section>
        <div class="section-head">
          <span class="section-label">Your Library</span>
          <span class="count">{$workspaceBooks.length} {$workspaceBooks.length === 1 ? "book" : "books"}</span>
        </div>
        <div class="grid">
          {#each $workspaceBooks as book (book.root)}
            <button class="book-card hover-lift" onclick={() => openProjectFolder(book.root)}>
              <div class="spine" style="background: linear-gradient(135deg, #a86b2c, color-mix(in srgb, #a86b2c 70%, #000))">
                <div class="spine-overlay"></div>
                <div class="spine-content">
                  <span class="genre">{book.meta.language || "Fiction"}</span>
                  <h4 class="book-title">{book.meta.title}</h4>
                  <p class="author">by {book.meta.author}</p>
                </div>
              </div>
              <div class="body">
                <div class="meta-row">
                  <span class="chapters">{book.chapters.length} {book.chapters.length === 1 ? "chapter" : "chapters"}</span>
                  <span class="dot">·</span>
                  <span class="updated">{formatRelative(book.meta.updatedAt)}</span>
                </div>
                <div class="words">{formatNumber(book.chapters.reduce((s, c) => s + c.wordCount, 0))} words total</div>
              </div>
            </button>
          {/each}
        </div>
      </section>
    {/if}
  </main>
</div>

<style>
  .library-view {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-0);
  }
  .header {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 28px;
    background: color-mix(in srgb, var(--bg-0) 88%, transparent);
    backdrop-filter: blur(8px);
    border-bottom: 1px solid var(--border-soft);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background: var(--accent-soft);
    color: var(--paper);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    font-family: var(--serif);
  }
  .title {
    margin: 0;
    font-family: var(--serif);
    font-size: 19px;
    font-weight: 500;
    color: var(--fg-0);
    letter-spacing: -0.01em;
  }
  .tagline {
    margin: 0;
    font-size: 11px;
    color: var(--fg-2);
  }
  .new-btn {
    font-size: 13px;
  }
  .main {
    max-width: 1000px;
    margin: 0 auto;
    padding: 36px 28px;
  }
  .greeting {
    margin-bottom: 32px;
  }
  .greet {
    margin: 0;
    font-family: var(--serif);
    font-size: 32px;
    font-weight: 500;
    color: var(--fg-0);
    letter-spacing: -0.015em;
  }
  .sub {
    margin: 8px 0 0 0;
    font-family: var(--serif);
    font-size: 16px;
    color: var(--fg-1);
    line-height: 1.5;
  }
  .create-card {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 28px;
    max-width: 480px;
  }
  .create-card h3 {
    margin: 0 0 4px 0;
    font-family: var(--serif);
    font-size: 18px;
    color: var(--fg-0);
  }
  .create-card .hint {
    margin: 0 0 14px 0;
    font-size: 12px;
    color: var(--fg-2);
  }
  .create-card input {
    width: 100%;
    margin-bottom: 8px;
    font-size: 14px;
    padding: 8px 10px;
  }
  .create-card input:first-of-type {
    font-family: var(--serif);
    font-size: 16px;
  }
  .goal-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 10px 0 14px;
  }
  .goal-row label {
    font-size: 12px;
    color: var(--fg-1);
  }
  .goal-row input {
    width: 80px;
    margin: 0;
  }
  .goal-hint {
    font-size: 11px;
    color: var(--fg-2);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .empty {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 0;
    text-align: center;
  }
  .empty-glow {
    position: absolute;
    width: 256px;
    height: 256px;
    border-radius: 50%;
    margin-bottom: -160px;
    background: radial-gradient(circle at center, var(--accent-soft) 0%, transparent 70%);
    opacity: 0.18;
  }
  .empty-content {
    position: relative;
    z-index: 1;
  }
  .empty-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--paper);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 26px;
    margin: 0 auto 20px;
  }
  .empty h3 {
    margin: 0 0 8px 0;
    font-family: var(--serif);
    font-size: 22px;
    color: var(--fg-0);
  }
  .empty p {
    margin: 0 0 24px 0;
    font-family: var(--serif);
    font-size: 14px;
    color: var(--fg-1);
    line-height: 1.6;
  }
  .big {
    font-size: 14px;
    padding: 10px 18px;
  }
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
  }
  .section-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-2);
    font-weight: 600;
  }
  .count {
    font-size: 12px;
    color: var(--fg-2);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 16px;
  }
  .book-card {
    text-align: left;
    background: var(--bg-3);
    border: 1px solid var(--border-soft);
    border-radius: 12px;
    overflow: hidden;
    cursor: pointer;
    padding: 0;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }
  .book-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 24px -12px rgba(0, 0, 0, 0.25);
  }
  .spine {
    position: relative;
    height: 96px;
    padding: 14px;
    color: white;
    overflow: hidden;
  }
  .spine-overlay {
    position: absolute;
    inset: 0;
    background: radial-gradient(circle at 20% 20%, rgba(255, 255, 255, 0.3) 0%, transparent 50%);
    mix-blend-mode: overlay;
  }
  .spine-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    height: 100%;
  }
  .genre {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.8;
  }
  .book-title {
    margin: 0;
    font-family: var(--serif);
    font-weight: 500;
    font-size: 16px;
    line-height: 1.2;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .author {
    margin: 4px 0 0 0;
    font-size: 11px;
    opacity: 0.85;
  }
  .body {
    padding: 12px 14px;
  }
  .meta-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--fg-2);
  }
  .dot {
    color: var(--fg-2);
  }
  .words {
    margin-top: 4px;
    font-size: 11px;
    color: var(--fg-2);
    font-variant-numeric: tabular-nums;
  }
</style>
