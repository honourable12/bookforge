<script lang="ts">
  import { activeChapter, modelLoaded, modelStatus, project, toast, isGenerating } from "$lib/stores";
</script>

<div class="statusbar">
  <div class="left">
    {#if $project}
      <span class="item">{$project.meta.title}</span>
      <span class="sep">·</span>
      <span class="item">{$project.chapters.length} chapters</span>
      <span class="sep">·</span>
      <span class="item mono">{$project.root}</span>
    {:else}
      <span class="item muted">No project open</span>
    {/if}
  </div>
  <div class="right">
    {#if $activeChapter}
      <span class="item">{$activeChapter.title} ({$activeChapter.wordCount} words)</span>
      <span class="sep">·</span>
    {/if}
    {#if $isGenerating}
      <span class="item accent">● generating</span>
      <span class="sep">·</span>
    {/if}
    <span class="item" class:ok={$modelLoaded} class:bad={!$modelLoaded}>
      {$modelStatus}
    </span>
  </div>
</div>

{#if $toast}
  <div class="toast {$toast.kind}">
    {$toast.msg}
  </div>
{/if}

<style>
  .statusbar {
    height: 24px;
    background: var(--bg-2);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    font-size: 11px;
    color: var(--fg-1);
    flex-shrink: 0;
  }
  .left,
  .right {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
  }
  .item {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
  }
  .item.muted {
    color: var(--fg-2);
  }
  .item.mono {
    font-family: var(--mono);
    color: var(--fg-2);
  }
  .item.accent {
    color: var(--accent);
  }
  .item.ok {
    color: var(--success);
  }
  .item.bad {
    color: var(--warn);
  }
  .sep {
    color: var(--fg-2);
  }
  .toast {
    position: fixed;
    bottom: 36px;
    right: 16px;
    padding: 10px 16px;
    border-radius: 6px;
    font-size: 12px;
    z-index: 1000;
    max-width: 360px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    animation: slidein 0.18s ease-out;
  }
  .toast.info {
    background: var(--bg-3);
    border: 1px solid var(--border);
    color: var(--fg-0);
  }
  .toast.success {
    background: rgba(110, 231, 183, 0.15);
    border: 1px solid var(--success);
    color: var(--success);
  }
  .toast.error {
    background: rgba(248, 113, 113, 0.15);
    border: 1px solid var(--error);
    color: var(--error);
  }
  @keyframes slidein {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
