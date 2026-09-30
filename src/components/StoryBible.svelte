<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import {
    project,
    storyBible,
    refreshStoryBible,
    activeBibleTab,
    notify,
  } from "$lib/stores";
  import { api } from "$lib/api";
  import type { Character, Location, StoryNote, StoryNoteType } from "$lib/types";
  import EmptyState from "./EmptyState.svelte";

  let mounted = true;
  onMount(async () => {
    await refreshStoryBible();
  });
  onDestroy(() => {
    mounted = false;
  });

  // ----- new-entity dialog state -----
  let editing = $state<{
    kind: "character" | "location" | "note";
    id?: string;
  } | null>(null);

  // Character form
  let cName = $state("");
  let cRole = $state("");
  let cDescription = $state("");
  let cTraits = $state("");
  let cBackstory = $state("");
  let cAppearance = $state("");
  let cColor = $state("#b48cff");

  // Location form
  let lName = $state("");
  let lMood = $state("");
  let lDescription = $state("");
  let lColor = $state("#6ee7b7");

  // Note form
  let nTitle = $state("");
  let nType = $state<StoryNoteType>("outline");
  let nContent = $state("");
  let nColor = $state("#fbbf24");

  const CHARACTER_COLORS = ["#b48cff", "#fbbf24", "#6ee7b7", "#7c9cff", "#f87171", "#f9a8d4"];

  function resetForms() {
    cName = cRole = cDescription = cTraits = cBackstory = cAppearance = "";
    cColor = "#b48cff";
    lName = lMood = lDescription = "";
    lColor = "#6ee7b7";
    nTitle = nContent = "";
    nType = "outline";
    nColor = "#fbbf24";
  }

  function openEditor(kind: "character" | "location" | "note", id?: string) {
    editing = { kind, id };
    if (id) {
      const bible = get(storyBible);
      if (kind === "character") {
        const c = bible.characters.find((x) => x.id === id);
        if (c) {
          cName = c.name;
          cRole = c.role;
          cDescription = c.description;
          cTraits = c.traits;
          cBackstory = c.backstory;
          cAppearance = c.appearance;
          cColor = c.color;
        }
      } else if (kind === "location") {
        const l = bible.locations.find((x) => x.id === id);
        if (l) {
          lName = l.name;
          lMood = l.mood;
          lDescription = l.description;
          lColor = l.color;
        }
      } else {
        const n = bible.notes.find((x) => x.id === id);
        if (n) {
          nTitle = n.title;
          nType = n.type as StoryNoteType;
          nContent = n.content;
          nColor = n.color;
        }
      }
    } else {
      resetForms();
    }
  }

  async function saveEntity() {
    if (!editing || !$project) return;
    const root = $project.root;

    try {
      if (editing.kind === "character") {
        if (!cName.trim()) {
          notify("error", "Name is required.");
          return;
        }
        const c: Character = {
          id: editing.id ?? "",
          name: cName.trim(),
          role: cRole.trim(),
          description: cDescription.trim(),
          traits: cTraits.trim(),
          backstory: cBackstory.trim(),
          appearance: cAppearance.trim(),
          color: cColor,
        };
        await api.bibleUpsertCharacter(root, c);
      } else if (editing.kind === "location") {
        if (!lName.trim()) {
          notify("error", "Name is required.");
          return;
        }
        const l: Location = {
          id: editing.id ?? "",
          name: lName.trim(),
          mood: lMood.trim(),
          description: lDescription.trim(),
          color: lColor,
        };
        await api.bibleUpsertLocation(root, l);
      } else {
        if (!nTitle.trim()) {
          notify("error", "Title is required.");
          return;
        }
        const n: StoryNote = {
          id: editing.id ?? "",
          type: nType,
          title: nTitle.trim(),
          content: nContent,
          color: nColor,
        };
        await api.bibleUpsertNote(root, n);
      }
      await refreshStoryBible();
      notify("success", "Saved.");
      editing = null;
      resetForms();
    } catch (e) {
      notify("error", `Save failed: ${e}`);
    }
  }

  async function remove(kind: "character" | "location" | "note", id: string, name: string) {
    if (!confirm(`Delete "${name}"? This cannot be undone.`)) return;
    if (!$project) return;
    const root = $project.root;
    try {
      if (kind === "character") await api.bibleDeleteCharacter(root, id);
      else if (kind === "location") await api.bibleDeleteLocation(root, id);
      else await api.bibleDeleteNote(root, id);
      await refreshStoryBible();
      notify("success", "Deleted.");
    } catch (e) {
      notify("error", `Delete failed: ${e}`);
    }
  }

  let tabs = $derived([
    { id: "characters" as const, label: "Characters", count: $storyBible.characters.length, icon: "👤" },
    { id: "locations" as const, label: "Locations", count: $storyBible.locations.length, icon: "📍" },
    { id: "notes" as const, label: "Notes", count: $storyBible.notes.length, icon: "📝" },
  ]);
</script>

<div class="bible-view fade-in">
  <header class="header chrome-fade">
    <div>
      <h1>Story Bible</h1>
      <p>The world your characters live in. Reference it as you write.</p>
    </div>
    <div class="tabs">
      {#each tabs as t}
        <button
          class="tab"
          class:active={$activeBibleTab === t.id}
          onclick={() => activeBibleTab.set(t.id)}
        >
          <span class="tab-icon">{t.icon}</span>
          <span>{t.label}</span>
          <span class="count">{t.count}</span>
        </button>
      {/each}
    </div>
  </header>

  <div class="content">
    {#if $activeBibleTab === "characters"}
      {#if $storyBible.characters.length === 0}
        <EmptyState
          title="No characters yet"
          subtitle="Your protagonists, antagonists, foils, mentors — give them a home here."
          cta="Add a character"
          onAdd={() => openEditor("character")}
        />
      {:else}
        <div class="grid">
          {#each $storyBible.characters as c (c.id)}
            <div class="entity-card character hover-lift">
              <div class="card-head">
                <div class="avatar" style="background: {c.color}">
                  {c.name.charAt(0).toUpperCase()}
                </div>
                <div class="title-block">
                  <h4>{c.name}</h4>
                  {#if c.role}<p class="role">{c.role}</p>{/if}
                </div>
                <div class="card-actions">
                  <button class="icon-btn" onclick={() => openEditor("character", c.id)} title="Edit">✎</button>
                  <button class="icon-btn" onclick={() => remove("character", c.id, c.name)} title="Delete">×</button>
                </div>
              </div>
              {#if c.description}
                <p class="desc">{c.description}</p>
              {/if}
              {#if c.traits}
                <div class="traits">
                  {#each c.traits.split(",").map((t) => t.trim()).filter(Boolean).slice(0, 4) as t}
                    <span class="trait">{t}</span>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
          <button class="add-card" onclick={() => openEditor("character")}>
            <span>+</span><span>Add character</span>
          </button>
        </div>
      {/if}
    {:else if $activeBibleTab === "locations"}
      {#if $storyBible.locations.length === 0}
        <EmptyState
          title="No locations yet"
          subtitle="Villages, cities, rooms — the places where your story breathes."
          cta="Add a location"
          onAdd={() => openEditor("location")}
        />
      {:else}
        <div class="grid">
          {#each $storyBible.locations as l (l.id)}
            <div class="entity-card location hover-lift" style="border-top: 3px solid {l.color}">
              <div class="card-head">
                <div class="loc-icon" style="color: {l.color}">📍</div>
                <div class="title-block">
                  <h4>{l.name}</h4>
                  {#if l.mood}<p class="role italic">{l.mood}</p>{/if}
                </div>
                <div class="card-actions">
                  <button class="icon-btn" onclick={() => openEditor("location", l.id)} title="Edit">✎</button>
                  <button class="icon-btn" onclick={() => remove("location", l.id, l.name)} title="Delete">×</button>
                </div>
              </div>
              {#if l.description}
                <p class="desc">{l.description}</p>
              {/if}
            </div>
          {/each}
          <button class="add-card" onclick={() => openEditor("location")}>
            <span>+</span><span>Add location</span>
          </button>
        </div>
      {/if}
    {:else}
      {#if $storyBible.notes.length === 0}
        <EmptyState
          title="No notes yet"
          subtitle="Plot outlines, worldbuilding details, themes — capture them before they slip away."
          cta="Add a note"
          onAdd={() => openEditor("note")}
        />
      {:else}
        <div class="grid">
          {#each $storyBible.notes as n (n.id)}
            <div class="entity-card note hover-lift">
              <div class="card-head">
                <div class="note-dot" style="background: {n.color}"></div>
                <div class="title-block">
                  <h4>{n.title}</h4>
                  <p class="role">{n.type}</p>
                </div>
                <div class="card-actions">
                  <button class="icon-btn" onclick={() => openEditor("note", n.id)} title="Edit">✎</button>
                  <button class="icon-btn" onclick={() => remove("note", n.id, n.title)} title="Delete">×</button>
                </div>
              </div>
              {#if n.content}
                <p class="desc">{n.content}</p>
              {/if}
            </div>
          {/each}
          <button class="add-card" onclick={() => openEditor("note")}>
            <span>+</span><span>Add note</span>
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>

{#if editing}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <div class="modal-backdrop" role="dialog" tabindex="-1" onclick={() => (editing = null)}>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <div class="modal" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <h3>{editing.id ? "Edit" : "Add"} {editing.kind}</h3>
      {#if editing.kind === "character"}
        <label>Name<input bind:value={cName} placeholder="Lira Whittaker" /></label>
        <label>Role<input bind:value={cRole} placeholder="Protagonist · Mentor · Antagonist" /></label>
        <label>Description<textarea bind:value={cDescription} rows="3" placeholder="Who they are, what they want, what stands in their way."></textarea></label>
        <label>Traits <span class="hint">(comma-separated)</span><input bind:value={cTraits} placeholder="stubborn, loyal, afraid of water" /></label>
        <label>Backstory<textarea bind:value={cBackstory} rows="3" placeholder="Where they came from, what shaped them."></textarea></label>
        <label>Appearance<textarea bind:value={cAppearance} rows="2" placeholder="What the reader sees first."></textarea></label>
        <label>Color
          <div class="color-row">
            {#each CHARACTER_COLORS as c, ci}
              <button
                class="swatch"
                class:active={cColor === c}
                style="background: {c}"
                onclick={() => (cColor = c)}
                type="button"
                aria-label="Color {ci + 1}"
                title="Color {ci + 1}"
              ></button>
            {/each}
          </div>
        </label>
      {:else if editing.kind === "location"}
        <label>Name<input bind:value={lName} placeholder="The Saltmarsh" /></label>
        <label>Mood (one line)<input bind:value={lMood} placeholder="Cold, lonely, salt on the wind." class="italic" /></label>
        <label>Description<textarea bind:value={lDescription} rows="6" placeholder="What does this place look, sound, smell like?"></textarea></label>
      {:else}
        <label>Title<input bind:value={nTitle} placeholder="Three-act structure" /></label>
        <label>Type
          <div class="type-row">
            {#each ["outline", "worldbuilding", "plot", "theme"] as t}
              <button class="type-pill" class:active={nType === t} onclick={() => (nType = t as StoryNoteType)} type="button">{t}</button>
            {/each}
          </div>
        </label>
        <label>Content<textarea bind:value={nContent} rows="8" placeholder="Write freely. Markdown welcome."></textarea></label>
      {/if}
      <div class="modal-actions">
        <button onclick={() => (editing = null)}>Cancel</button>
        <button class="primary" onclick={saveEntity}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .bible-view {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-2);
  }
  .header {
    position: sticky;
    top: 0;
    z-index: 5;
    padding: 18px 28px;
    background: color-mix(in srgb, var(--bg-2) 92%, transparent);
    backdrop-filter: blur(8px);
    border-bottom: 1px solid var(--border-soft);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .header h1 {
    margin: 0;
    font-family: var(--serif);
    font-size: 22px;
    font-weight: 500;
    color: var(--fg-0);
  }
  .header p {
    margin: 4px 0 0 0;
    font-size: 12px;
    color: var(--fg-2);
  }
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: transparent;
    border: 1px solid var(--border-soft);
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    color: var(--fg-1);
  }
  .tab.active {
    background: var(--accent-soft);
    color: var(--paper);
    border-color: var(--accent-soft);
  }
  .tab-icon {
    font-size: 13px;
  }
  .count {
    background: var(--bg-3);
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-variant-numeric: tabular-nums;
  }
  .tab.active .count {
    background: rgba(0, 0, 0, 0.2);
    color: var(--paper);
  }
  .content {
    max-width: 1000px;
    margin: 0 auto;
    padding: 24px 28px 60px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 14px;
  }
  .entity-card {
    background: var(--bg-3);
    border: 1px solid var(--border-soft);
    border-radius: 10px;
    padding: 14px;
  }
  .card-head {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin-bottom: 8px;
  }
  .avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--serif);
    font-weight: 500;
    font-size: 16px;
    flex-shrink: 0;
  }
  .loc-icon {
    font-size: 20px;
    flex-shrink: 0;
  }
  .note-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    margin-top: 8px;
    flex-shrink: 0;
  }
  .title-block {
    flex: 1;
    min-width: 0;
  }
  .title-block h4 {
    margin: 0;
    font-family: var(--serif);
    font-size: 15px;
    font-weight: 500;
    color: var(--fg-0);
    line-height: 1.2;
  }
  .role {
    margin: 2px 0 0 0;
    font-size: 11px;
    color: var(--fg-2);
  }
  .role.italic {
    font-style: italic;
    font-family: var(--serif);
  }
  .card-actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .entity-card:hover .card-actions {
    opacity: 1;
  }
  .icon-btn {
    background: transparent;
    border: none;
    color: var(--fg-2);
    width: 22px;
    height: 22px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    padding: 0;
  }
  .icon-btn:hover {
    background: var(--bg-2);
    color: var(--fg-0);
  }
  .desc {
    margin: 6px 0 0 0;
    font-family: var(--serif);
    font-size: 13px;
    line-height: 1.5;
    color: var(--fg-1);
    display: -webkit-box;
    -webkit-line-clamp: 4;
    -webkit-box-orient: vertical;
    overflow: hidden;
    white-space: pre-wrap;
  }
  .traits {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 8px;
  }
  .trait {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--bg-2);
    border-radius: 3px;
    color: var(--fg-1);
  }
  .add-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 2px dashed var(--border);
    background: transparent;
    color: var(--fg-2);
    cursor: pointer;
    padding: 28px;
    border-radius: 10px;
    min-height: 120px;
  }
  .add-card:hover {
    background: var(--bg-3);
    border-color: var(--accent-soft);
    color: var(--fg-0);
  }
  .add-card span:first-child {
    font-size: 20px;
  }
  .add-card span:last-child {
    font-family: var(--serif);
    font-size: 13px;
  }

  /* Modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }
  .modal {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    max-width: 480px;
    width: 100%;
    max-height: 80vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .modal h3 {
    margin: 0 0 4px 0;
    font-family: var(--serif);
    font-size: 17px;
    color: var(--fg-0);
    text-transform: capitalize;
  }
  .modal label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--fg-2);
    font-weight: 500;
  }
  .modal .hint {
    text-transform: none;
    font-size: 10px;
    color: var(--fg-2);
    margin-left: 4px;
  }
  .modal input,
  .modal textarea {
    width: 100%;
    font-family: var(--sans);
    font-size: 13px;
    padding: 7px 10px;
    text-transform: none;
    letter-spacing: normal;
    font-weight: 400;
    color: var(--fg-0);
  }
  .modal input.italic {
    font-style: italic;
    font-family: var(--serif);
  }
  .modal textarea {
    font-family: var(--serif);
    line-height: 1.5;
    resize: vertical;
  }
  .color-row,
  .type-row {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .swatch {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
  }
  .swatch.active {
    border-color: var(--fg-0);
  }
  .type-pill {
    background: var(--bg-3);
    border: 1px solid var(--border-soft);
    color: var(--fg-1);
    padding: 4px 10px;
    border-radius: 12px;
    cursor: pointer;
    font-size: 11px;
    text-transform: lowercase;
  }
  .type-pill.active {
    background: var(--accent);
    color: var(--paper);
    border-color: var(--accent);
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }
</style>
