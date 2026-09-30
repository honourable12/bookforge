<script lang="ts">
  import {
    activeChapter,
    modelLoaded,
    modelStatus,
    project,
    toast,
    isGenerating,
    writingStats,
    view,
  } from "$lib/stores";
  import { formatNumber } from "$lib/util";

  let todayWords = $derived($writingStats?.todayWords ?? 0);
  let dailyGoal = $derived($writingStats?.dailyGoal ?? 500);
  let goalMet = $derived($writingStats?.todayGoalMet ?? false);
  let streak = $derived($writingStats?.currentStreak ?? 0);
  let totalWords = $derived($writingStats?.totalWords ?? 0);
</script>

<div class="statusbar">
  <div class="left">
    {#if $project}
      <button class="item link" onclick={() => view.set("library")}>
        ← Library
      </button>
      <span class="sep">·</span>
      <span class="item">{$project.meta.title}</span>
      <span class="sep">·</span>
      <span class="item muted">{formatNumber(totalWords)} words total</span>
    {:else}
      <button class="item link" onclick={() => view.set("library")}>
        ← Back to Library
      </button>
    {/if}
  </div>
  <div class="right">
    {#if $activeChapter}
      <span class="item">{$activeChapter.title}</span>
      <span class="sep">·</span>
      <span class="item">{formatNumber($activeChapter.wordCount)} words</span>
      <span class="sep">·</span>
    {/if}
    {#if dailyGoal > 0}
      <span class="item" class:met={goalMet}>
        {#if goalMet}✓{:else}◐{/if}
        {formatNumber(todayWords)} / {formatNumber(dailyGoal)} today
      </span>
      <span class="sep">·</span>
    {/if}
    {#if streak > 0}
      <span class="item streak">🔥 {streak}-day streak</span>
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
    height: 22px;
    background: var(--bg-2);
    border-top: 1px solid var(--border-soft);
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
    max-width: 320px;
    font-variant-numeric: tabular-nums;
  }
  .item.muted {
    color: var(--fg-2);
  }
  .item.link {
    background: transparent;
    border: none;
    color: var(--accent);
    cursor: pointer;
    padding: 0;
    font-size: 11px;
  }
  .item.link:hover {
    text-decoration: underline;
  }
  .item.met {
    color: var(--success);
  }
  .item.streak {
    color: var(--warn);
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
    bottom: 30px;
    right: 16px;
    padding: 10px 16px;
    border-radius: 6px;
    font-size: 12px;
    z-index: 1000;
    max-width: 360px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    animation: slidein 0.18s ease-out;
  }
  .toast.info {
    background: var(--bg-3);
    border: 1px solid var(--border);
    color: var(--fg-0);
  }
  .toast.success {
    background: color-mix(in srgb, var(--success) 18%, var(--bg-3));
    border: 1px solid var(--success);
    color: var(--success);
  }
  .toast.error {
    background: color-mix(in srgb, var(--error) 18%, var(--bg-3));
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
