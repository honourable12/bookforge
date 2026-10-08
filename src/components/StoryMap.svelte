<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import {
    project,
    storyBible,
    storyMap,
    refreshStoryBible,
    refreshStoryMap,
    saveStoryMap,
    notify,
  } from "$lib/stores";
  import { api } from "$lib/api";
  import type {
    StoryMap,
    MapNode,
    MapEdge,
    MapNote,
    MapNodeKind,
  } from "$lib/types";
  import { COMMON_EDGE_KINDS } from "$lib/types";
  import { uuid } from "$lib/util";
  import EmptyState from "./EmptyState.svelte";

  let canvas: SVGSVGElement;
  let showAddNode = $state(false);
  let showAddNote = $state(false);
  let showContextPreview = $state(false);
  let contextPreview = $state("");
  let dragging = $state<{ kind: "node" | "note"; id: string; offX: number; offY: number } | null>(null);
  let connecting = $state<{ fromId: string; x: number; y: number } | null>(null);
  let selectedEdge = $state<MapEdge | null>(null);
  let edgeKindDraft = $state("allies_with");
  let edgeNoteDraft = $state("");

  // New note form
  let newNoteTitle = $state("");
  let newNoteContent = $state("");

  onMount(async () => {
    await refreshStoryBible();
    await refreshStoryMap();
  });

  // ----- canvas geometry -----
  const WIDTH = 1000;
  const HEIGHT = 640;

  function centerPosition(idx: number): { x: number; y: number } {
    // Place new nodes in a loose spiral around center so they don't stack.
    const cx = WIDTH / 2;
    const cy = HEIGHT / 2;
    const angle = idx * 0.9;
    const radius = 80 + idx * 28;
    return {
      x: cx + radius * Math.cos(angle),
      y: cy + radius * Math.sin(angle),
    };
  }

  // ----- add a node from a Story Bible entity -----
  function addNode(kind: MapNodeKind, refId: string, label: string, color: string) {
    const m = get(storyMap);
    // Don't duplicate nodes for the same entity.
    if (m.nodes.find((n) => n.refId === refId)) {
      notify("info", `${label} is already on the map.`);
      return;
    }
    const pos = centerPosition(m.nodes.length);
    const node: MapNode = { kind, refId, label, color, x: pos.x, y: pos.y };
    const next = { ...m, nodes: [...m.nodes, node] };
    saveStoryMap(next);
  }

  function removeNode(refId: string) {
    const m = get(storyMap);
    const next = {
      ...m,
      nodes: m.nodes.filter((n) => n.refId !== refId),
      // Remove edges that referenced this node.
      edges: m.edges.filter((e) => e.fromId !== refId && e.toId !== refId),
    };
    saveStoryMap(next);
  }

  // ----- dragging nodes -----
  function onNodeMouseDown(e: MouseEvent, node: MapNode) {
    e.preventDefault();
    e.stopPropagation();
    const rect = canvas.getBoundingClientRect();
    const scaleX = WIDTH / rect.width;
    const scaleY = HEIGHT / rect.height;
    const mx = (e.clientX - rect.left) * scaleX;
    const my = (e.clientY - rect.top) * scaleY;
    dragging = { kind: "node", id: node.refId, offX: mx - node.x, offY: my - node.y };
  }

  function onNoteMouseDown(e: MouseEvent, note: MapNote) {
    e.preventDefault();
    e.stopPropagation();
    const rect = canvas.getBoundingClientRect();
    const scaleX = WIDTH / rect.width;
    const scaleY = HEIGHT / rect.height;
    const mx = (e.clientX - rect.left) * scaleX;
    const my = (e.clientY - rect.top) * scaleY;
    dragging = { kind: "note", id: note.id, offX: mx - note.x, offY: my - note.y };
  }

  function onMouseMove(e: MouseEvent) {
    if (!dragging) return;
    const rect = canvas.getBoundingClientRect();
    const scaleX = WIDTH / rect.width;
    const scaleY = HEIGHT / rect.height;
    const mx = (e.clientX - rect.left) * scaleX;
    const my = (e.clientY - rect.top) * scaleY;
    const m = get(storyMap);
    if (dragging.kind === "node") {
      const next = {
        ...m,
        nodes: m.nodes.map((n) =>
          n.refId === dragging!.id
            ? { ...n, x: Math.max(40, Math.min(WIDTH - 40, mx - dragging!.offX)),
                       y: Math.max(30, Math.min(HEIGHT - 30, my - dragging!.offY)) }
            : n,
        ),
      };
      storyMap.set(next);
    } else {
      const next = {
        ...m,
        notes: m.notes.map((n) =>
          n.id === dragging!.id
            ? { ...n, x: Math.max(40, Math.min(WIDTH - 100, mx - dragging!.offX)),
                       y: Math.max(30, Math.min(HEIGHT - 60, my - dragging!.offY)) }
            : n,
        ),
      };
      storyMap.set(next);
    }
  }

  function onMouseUp() {
    if (dragging) {
      // Persist final positions.
      saveStoryMap(get(storyMap));
      dragging = null;
    }
    if (connecting) {
      // Mouse up without landing on a node — cancel the connection.
      connecting = null;
    }
  }

  // ----- connecting nodes with edges -----
  function onNodeConnectStart(e: MouseEvent, node: MapNode) {
    e.preventDefault();
    e.stopPropagation();
    const rect = canvas.getBoundingClientRect();
    const scaleX = WIDTH / rect.width;
    const scaleY = HEIGHT / rect.height;
    connecting = {
      fromId: node.refId,
      x: (e.clientX - rect.left) * scaleX,
      y: (e.clientY - rect.top) * scaleY,
    };
  }

  function onCanvasMouseMove(e: MouseEvent) {
    if (!connecting) {
      onMouseMove(e);
      return;
    }
    const rect = canvas.getBoundingClientRect();
    const scaleX = WIDTH / rect.width;
    const scaleY = HEIGHT / rect.height;
    connecting = {
      ...connecting,
      x: (e.clientX - rect.left) * scaleX,
      y: (e.clientY - rect.top) * scaleY,
    };
  }

  function onNodeConnectEnd(node: MapNode) {
    if (!connecting) return;
    if (connecting.fromId === node.refId) {
      connecting = null;
      return;
    }
    const m = get(storyMap);
    
    const exists = m.edges.find(
      (e) =>
        e.fromId === connecting!.fromId &&
        e.toId === node.refId &&
        e.kind === edgeKindDraft,
    );
    if (exists) {
      notify("info", "That exact relationship already exists.");
      connecting = null;
      return;
    }
    const edge: MapEdge = {
      id: uuid(),
      fromId: connecting.fromId,
      toId: node.refId,
      kind: edgeKindDraft,
      note: edgeNoteDraft.trim(),
    };
    const next = { ...m, edges: [...m.edges, edge] };
    saveStoryMap(next);
    connecting = null;
    edgeNoteDraft = "";
  }

  function removeEdge(id: string) {
    const m = get(storyMap);
    saveStoryMap({ ...m, edges: m.edges.filter((e) => e.id !== id) });
    selectedEdge = null;
  }

  function selectEdge(e: MapEdge) {
    selectedEdge = e;
    edgeKindDraft = e.kind;
    edgeNoteDraft = e.note;
  }

  // ----- map notes -----
  function addNote() {
    if (!newNoteTitle.trim()) return;
    const m = get(storyMap);
    const pos = centerPosition(m.notes.length + 1);
    const note: MapNote = {
      id: uuid(),
      title: newNoteTitle.trim(),
      content: newNoteContent.trim(),
      color: "#fbbf24",
      x: pos.x,
      y: pos.y,
    };
    saveStoryMap({ ...m, notes: [...m.notes, note] });
    newNoteTitle = "";
    newNoteContent = "";
    showAddNote = false;
  }

  function removeNote(id: string) {
    const m = get(storyMap);
    saveStoryMap({ ...m, notes: m.notes.filter((n) => n.id !== id) });
  }

  // ----- context preview (what the LLM will see) -----
  async function previewContext() {
    const p = get(project);
    if (!p) return;
    try {
      const ctx = await api.storyMapPreviewContext(p.root);
      contextPreview = ctx || "(map is empty — add nodes to give the LLM context)";
      showContextPreview = true;
    } catch (e) {
      notify("error", `Preview failed: ${e}`);
    }
  }

  // ----- helpers -----
  function nodeByRefId(refId: string): MapNode | undefined {
    return get(storyMap).nodes.find((n) => n.refId === refId);
  }

  // Available bible entities not yet on the map.
  let availableCharacters = $derived(
    $storyBible.characters.filter(
      (c) => !$storyMap.nodes.find((n) => n.refId === c.id),
    ),
  );
  let availableLocations = $derived(
    $storyBible.locations.filter(
      (l) => !$storyMap.nodes.find((n) => n.refId === l.id),
    ),
  );
  let availableNotes = $derived(
    $storyBible.notes.filter(
      (n) => !$storyMap.nodes.find((m) => m.refId === n.id),
    ),
  );

  let isEmpty = $derived(
    $storyMap.nodes.length === 0 &&
      $storyMap.edges.length === 0 &&
      $storyMap.notes.length === 0,
  );
</script>

<div class="map-view fade-in">
  <header class="header chrome-fade">
    <div>
      <h1>Story Map</h1>
      <p>The graph of who's who and how they relate. The AI uses this as canon when continuing or rewriting your prose.</p>
    </div>
    <div class="actions">
      <button onclick={() => (showAddNode = !showAddNode)}>+ Add entity</button>
      <button onclick={() => (showAddNote = !showAddNote)}>+ Add note</button>
      <button onclick={previewContext} title="Preview the text block the AI will see">Preview AI context</button>
    </div>
  </header>

  {#if showAddNode}
    <div class="add-panel">
      <h3>Add to map</h3>
      {#if availableCharacters.length === 0 && availableLocations.length === 0 && availableNotes.length === 0}
        <p class="hint">All Story Bible entities are already on the map. Add more in the Story Bible view.</p>
      {/if}
      {#if availableCharacters.length > 0}
        <div class="add-group">
          <div class="add-group-label">Characters</div>
          {#each availableCharacters as c}
            <button class="add-pill" style="border-color: {c.color}" onclick={() => addNode("character", c.id, c.name, c.color)}>
              <span class="dot" style="background: {c.color}"></span>
              {c.name}
            </button>
          {/each}
        </div>
      {/if}
      {#if availableLocations.length > 0}
        <div class="add-group">
          <div class="add-group-label">Locations</div>
          {#each availableLocations as l}
            <button class="add-pill" style="border-color: {l.color}" onclick={() => addNode("location", l.id, l.name, l.color)}>
              <span class="dot" style="background: {l.color}"></span>
              {l.name}
            </button>
          {/each}
        </div>
      {/if}
      {#if availableNotes.length > 0}
        <div class="add-group">
          <div class="add-group-label">Notes</div>
          {#each availableNotes as n}
            <button class="add-pill" style="border-color: {n.color}" onclick={() => addNode("note", n.id, n.title, n.color)}>
              <span class="dot" style="background: {n.color}"></span>
              {n.title}
            </button>
          {/each}
        </div>
      {/if}
      <button class="close-panel" onclick={() => (showAddNode = false)}>Done</button>
    </div>
  {/if}

  {#if showAddNote}
    <div class="add-panel">
      <h3>Add theme / conflict note</h3>
      <input bind:value={newNoteTitle} placeholder="Title (e.g. 'The central question')" />
      <textarea bind:value={newNoteContent} rows="2" placeholder="What does the reader need to remember?"></textarea>
      <div class="row">
        <button class="primary" onclick={addNote} disabled={!newNoteTitle.trim()}>Add</button>
        <button onclick={() => (showAddNote = false)}>Cancel</button>
      </div>
    </div>
  {/if}

  {#if showContextPreview}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <div class="modal-backdrop" role="dialog" tabindex="-1" onclick={() => (showContextPreview = false)}>
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
      <div class="modal" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
        <h3>What the AI sees</h3>
        <p class="hint">This text block is injected into the system prompt whenever you click "Continue" or "Rewrite" in the editor.</p>
        <pre class="context-preview">{contextPreview}</pre>
        <div class="modal-actions">
          <button onclick={() => (showContextPreview = false)}>Close</button>
        </div>
      </div>
    </div>
  {/if}

  {#if isEmpty && !showAddNode}
    <EmptyState
      title="Your story map is empty"
      subtitle="Add characters, locations, and plot notes from your Story Bible, then draw relationships between them. The AI will consult this graph when continuing your prose."
      cta="Add an entity"
      onAdd={() => (showAddNode = true)}
    />
  {:else}
    <!-- Edge kind selector (used when drawing new edges) -->
    <div class="edge-toolbar">
      <span class="edge-label">New relationship:</span>
      <select
        value={edgeKindDraft}
        onchange={(e) => (edgeKindDraft = (e.currentTarget as HTMLSelectElement).value)}
      >
        {#each COMMON_EDGE_KINDS as k}
          <option value={k} selected={k === edgeKindDraft}>{k.replace(/_/g, " ")}</option>
        {/each}
      </select>
      <input
        value={edgeNoteDraft}
        oninput={(e) => (edgeNoteDraft = (e.currentTarget as HTMLInputElement).value)}
        placeholder="optional note"
        class="edge-note-input"
      />
      <span class="edge-hint">Drag from a node's ◉ to another to connect them.</span>
    </div>

    <div class="canvas-wrap">
      <svg
        bind:this={canvas}
        class="canvas"
        viewBox="0 0 {WIDTH} {HEIGHT}"
        preserveAspectRatio="xMidYMid meet"
        onmousemove={onCanvasMouseMove}
        onmouseup={onMouseUp}
        onmouseleave={onMouseUp}
      >
        <!-- Grid background -->
        <defs>
          <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
            <path d="M 40 0 L 0 0 0 40" fill="none" stroke="var(--border-soft)" stroke-width="0.5" />
          </pattern>
        </defs>
        <rect width={WIDTH} height={HEIGHT} fill="url(#grid)" />

        <!-- Edges -->
        {#each $storyMap.edges as edge (edge.id)}
          {@const from = nodeByRefId(edge.fromId)}
          {@const to = nodeByRefId(edge.toId)}
          {#if from && to}
            {@const midX = (from.x + to.x) / 2}
            {@const midY = (from.y + to.y) / 2}
            <line
              x1={from.x}
              y1={from.y}
              x2={to.x}
              y2={to.y}
              stroke={selectedEdge?.id === edge.id ? "var(--accent)" : "var(--fg-2)"}
              stroke-width={selectedEdge?.id === edge.id ? 2.5 : 1.5}
              marker-end="url(#arrowhead)"
              class="edge-line"
              role="button"
              tabindex="0"
              onclick={() => selectEdge(edge)}
              onkeydown={(e) => e.key === "Enter" && selectEdge(edge)}
            />
            <text
              x={midX}
              y={midY - 6}
              text-anchor="middle"
              class="edge-label-text"
              onclick={() => selectEdge(edge)}
              onkeydown={(e) => e.key === "Enter" && selectEdge(edge)}
              role="button"
              tabindex="0"
            >{edge.kind.replace(/_/g, " ")}</text>
          {/if}
        {/each}

        <!-- In-progress connection line -->
        {#if connecting}
          {@const from = nodeByRefId(connecting.fromId)}
          {#if from}
            <line
              x1={from.x}
              y1={from.y}
              x2={connecting.x}
              y2={connecting.y}
              stroke="var(--accent)"
              stroke-width="2"
              stroke-dasharray="4 4"
              pointer-events="none"
            />
          {/if}
        {/if}

        <!-- Arrowhead marker -->
        <defs>
          <marker
            id="arrowhead"
            markerWidth="8"
            markerHeight="8"
            refX="6"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 8 3, 0 6" fill="var(--fg-2)" />
          </marker>
        </defs>

        <!-- Map notes (drawn under nodes) -->
        {#each $storyMap.notes as note (note.id)}
          <g
            class="map-note"
            onmousedown={(e) => onNoteMouseDown(e, note)}
            transform="translate({note.x}, {note.y})"
          >
            <rect
              x="-70"
              y="-22"
              width="140"
              height="44"
              rx="6"
              fill="var(--bg-3)"
              stroke={note.color}
              stroke-width="2"
            />
            <text x="0" y="-4" text-anchor="middle" class="note-title">{note.title}</text>
            <text x="0" y="12" text-anchor="middle" class="note-content">{note.content.slice(0, 28)}{note.content.length > 28 ? "…" : ""}</text>
            <circle cx="62" cy="-14" r="7" fill="var(--bg-2)" class="note-close" onclick={() => removeNote(note.id)} role="button" tabindex="0" onkeydown={(e) => e.key === "Enter" && removeNote(note.id)} />
            <text x="62" y="-10" text-anchor="middle" class="note-close-x" onclick={() => removeNote(note.id)} role="button" tabindex="0" onkeydown={(e) => e.key === "Enter" && removeNote(note.id)}>×</text>
          </g>
        {/each}

        <!-- Nodes -->
        {#each $storyMap.nodes as node (node.refId)}
          <g
            class="map-node"
            onmousedown={(e) => onNodeMouseDown(e, node)}
            onmouseup={(e) => { e.stopPropagation(); onNodeConnectEnd(node); }}
            transform="translate({node.x}, {node.y})"
          >
            <!-- main circle -->
            <circle
              r="24"
              fill={node.color}
              stroke="var(--bg-1)"
              stroke-width="2"
            />
            <text
              x="0"
              y="5"
              text-anchor="middle"
              class="node-letter"
            >{node.label.charAt(0).toUpperCase()}</text>
            <!-- label below -->
            <text
              x="0"
              y="42"
              text-anchor="middle"
              class="node-label"
            >{node.label}</text>
            <!-- kind label -->
            <text
              x="0"
              y="56"
              text-anchor="middle"
              class="node-kind"
            >{node.kind}</text>
            <!-- connect handle (top-right) -->
            <circle
              cx="17"
              cy="-17"
              r="6"
              fill="var(--bg-3)"
              stroke={node.color}
              stroke-width="1.5"
              class="connect-handle"
              onmousedown={(e) => onNodeConnectStart(e, node)}
              onmouseup={() => onNodeConnectEnd(node)}
              role="button"
              tabindex="0"
              aria-label="Drag to connect {node.label}"
            />
            <text x="17" y="-14" text-anchor="middle" class="connect-handle-icon">◉</text>
            <!-- delete handle (top-left) -->
            <circle
              cx="-17"
              cy="-17"
              r="6"
              fill="var(--bg-3)"
              class="delete-handle"
              onclick={() => removeNode(node.refId)}
              role="button"
              tabindex="0"
              aria-label="Remove {node.label} from map"
              onkeydown={(e) => e.key === "Enter" && removeNode(node.refId)}
            />
            <text x="-17" y="-14" text-anchor="middle" class="delete-handle-icon">×</text>
          </g>
        {/each}
      </svg>
    </div>

    <!-- Selected edge detail panel -->
    {#if selectedEdge}
      {@const from = nodeByRefId(selectedEdge.fromId)}
      {@const to = nodeByRefId(selectedEdge.toId)}
      <div class="edge-detail">
        <strong>{from?.label ?? "?"}</strong>
        <span class="verb">{selectedEdge.kind.replace(/_/g, " ")}</span>
        <strong>{to?.label ?? "?"}</strong>
        {#if selectedEdge.note}<span class="edge-note-detail">— {selectedEdge.note}</span>{/if}
        <button class="danger" onclick={() => removeEdge(selectedEdge.id)}>Delete edge</button>
        <button onclick={() => (selectedEdge = null)}>Close</button>
      </div>
    {/if}

    <!-- Sidebar legend -->
    <div class="legend">
      <div class="legend-item"><span class="legend-dot" style="background: var(--accent)"></span> selected edge</div>
      <div class="legend-item"><span class="legend-dot" style="background: var(--fg-2)"></span> relationship</div>
      <div class="legend-item">◉ drag to connect</div>
      <div class="legend-item">× click to remove</div>
    </div>
  {/if}
</div>

<style>
  .map-view {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-2);
    display: flex;
    flex-direction: column;
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
    max-width: 540px;
  }
  .actions {
    display: flex;
    gap: 6px;
  }
  .add-panel {
    margin: 12px 28px 0;
    padding: 14px 16px;
    background: var(--bg-3);
    border: 1px solid var(--border-soft);
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .add-panel h3 {
    margin: 0;
    font-family: var(--serif);
    font-size: 15px;
    color: var(--fg-0);
  }
  .add-panel .hint {
    margin: 0;
    font-size: 12px;
    color: var(--fg-2);
  }
  .add-group {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .add-group-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-2);
    font-weight: 600;
    width: 100%;
  }
  .add-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    background: transparent;
    border: 1px solid;
    border-radius: 14px;
    font-size: 12px;
    color: var(--fg-1);
    cursor: pointer;
  }
  .add-pill:hover {
    background: var(--bg-2);
    color: var(--fg-0);
  }
  .add-pill .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .add-panel input,
  .add-panel textarea {
    width: 100%;
    font-size: 13px;
    padding: 6px 10px;
  }
  .add-panel textarea {
    font-family: var(--serif);
    resize: vertical;
  }
  .add-panel .row {
    display: flex;
    gap: 6px;
  }
  .close-panel {
    align-self: flex-end;
    font-size: 12px;
  }

  /* Edge toolbar */
  .edge-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 28px;
    background: var(--bg-1);
    border-bottom: 1px solid var(--border-soft);
    flex-shrink: 0;
    flex-wrap: wrap;
  }
  .edge-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-2);
    font-weight: 600;
  }
  .edge-toolbar select {
    font-size: 12px;
    padding: 3px 6px;
    background: var(--bg-3);
    color: var(--fg-0);
    border: 1px solid var(--border);
    border-radius: 4px;
  }
  .edge-note-input {
    width: 180px;
    font-size: 12px;
    padding: 3px 8px;
  }
  .edge-hint {
    font-size: 11px;
    color: var(--fg-2);
    font-style: italic;
  }

  /* Canvas */
  .canvas-wrap {
    flex: 1;
    padding: 12px 28px 28px;
    overflow: auto;
  }
  .canvas {
    width: 100%;
    height: 640px;
    background: var(--bg-1);
    border: 1px solid var(--border-soft);
    border-radius: 10px;
    cursor: default;
    user-select: none;
  }
  .map-node {
    cursor: grab;
  }
  .map-node:active {
    cursor: grabbing;
  }
  .node-letter {
    font-family: var(--serif);
    font-size: 16px;
    font-weight: 500;
    fill: white;
    pointer-events: none;
  }
  .node-label {
    font-family: var(--sans);
    font-size: 11px;
    font-weight: 500;
    fill: var(--fg-0);
    pointer-events: none;
  }
  .node-kind {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    fill: var(--fg-2);
    pointer-events: none;
  }
  .connect-handle,
  .delete-handle {
    cursor: pointer;
  }
  .connect-handle:hover {
    fill: var(--accent-soft);
  }
  .delete-handle:hover {
    fill: var(--error);
  }
  .connect-handle-icon,
  .delete-handle-icon {
    font-size: 9px;
    fill: var(--fg-1);
    pointer-events: none;
    user-select: none;
  }
  .delete-handle-icon {
    fill: var(--fg-1);
    font-size: 11px;
    font-weight: bold;
  }

  /* Map notes */
  .map-note {
    cursor: grab;
  }
  .map-note:active {
    cursor: grabbing;
  }
  .note-title {
    font-family: var(--serif);
    font-size: 11px;
    font-weight: 600;
    fill: var(--fg-0);
    pointer-events: none;
  }
  .note-content {
    font-size: 9px;
    fill: var(--fg-2);
    pointer-events: none;
  }
  .note-close {
    cursor: pointer;
  }
  .note-close:hover {
    fill: var(--error);
  }
  .note-close-x {
    font-size: 11px;
    font-weight: bold;
    fill: var(--fg-1);
    pointer-events: none;
  }

  /* Edges */
  .edge-line {
    cursor: pointer;
  }
  .edge-line:hover {
    stroke: var(--accent) !important;
    stroke-width: 2.5;
  }
  .edge-label-text {
    font-size: 9px;
    fill: var(--fg-1);
    cursor: pointer;
    text-transform: capitalize;
    font-family: var(--sans);
  }
  .edge-label-text:hover {
    fill: var(--accent);
  }

  /* Edge detail panel */
  .edge-detail {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 28px;
    background: var(--bg-3);
    border-top: 1px solid var(--border-soft);
    font-size: 12px;
    color: var(--fg-1);
  }
  .edge-detail .verb {
    color: var(--accent);
    font-style: italic;
    text-transform: capitalize;
  }
  .edge-note-detail {
    color: var(--fg-2);
  }
  .edge-detail button {
    margin-left: auto;
    font-size: 11px;
  }
  .danger {
    background: transparent;
    border: 1px solid var(--error);
    color: var(--error);
  }
  .danger:hover {
    background: var(--error);
    color: white;
  }

  /* Legend */
  .legend {
    display: flex;
    gap: 16px;
    padding: 8px 28px 16px;
    font-size: 10px;
    color: var(--fg-2);
    flex-wrap: wrap;
  }
  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .legend-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }

  /* Context preview modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.5);
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
    max-width: 640px;
    width: 100%;
    max-height: 80vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .modal h3 {
    margin: 0;
    font-family: var(--serif);
    font-size: 17px;
    color: var(--fg-0);
  }
  .modal .hint {
    margin: 0;
    font-size: 12px;
    color: var(--fg-2);
  }
  .context-preview {
    background: var(--bg-1);
    border: 1px solid var(--border-soft);
    border-radius: 6px;
    padding: 12px;
    font-family: var(--mono);
    font-size: 11px;
    line-height: 1.5;
    color: var(--fg-1);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 50vh;
    overflow-y: auto;
    margin: 0;
    -webkit-user-select: text;
    user-select: text;
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
