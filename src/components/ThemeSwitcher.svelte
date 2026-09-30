<script lang="ts">
  import { theme, setTheme } from "$lib/stores";
  import type { Theme } from "$lib/types";

  const themes: { id: Theme; label: string; preview: string }[] = [
    { id: "sepia", label: "Sepia", preview: "#f0e6d2" },
    { id: "light", label: "Light", preview: "#faf8f3" },
    { id: "dark", label: "Dark", preview: "#1a1612" },
    { id: "night", label: "Night", preview: "#0a0a0c" },
  ];

  let open = $state(false);

  function pick(t: Theme) {
    setTheme(t);
    open = false;
  }
</script>

<div class="theme-switcher">
  <button
    class="trigger"
    onclick={() => (open = !open)}
    title="Switch reading theme"
    aria-label="Switch theme"
  >
    <span class="dot" style="background: {themes.find((t) => t.id === $theme)?.preview}"></span>
    <span class="label">{$theme}</span>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <div
      class="menu"
      role="listbox"
      tabindex="-1"
      onfocusout={() => setTimeout(() => (open = false), 100)}
    >
      {#each themes as t}
        <button
          class="item"
          class:active={$theme === t.id}
          onclick={() => pick(t.id)}
          role="option"
          aria-selected={$theme === t.id}
        >
          <span class="swatch" style="background: {t.preview}"></span>
          <span class="name">{t.label}</span>
          {#if $theme === t.id}
            <span class="check">✓</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .theme-switcher {
    position: relative;
  }
  .trigger {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background: transparent;
    border: 1px solid var(--border-soft);
    border-radius: 5px;
    cursor: pointer;
    font-size: 11px;
    color: var(--fg-1);
    text-transform: capitalize;
  }
  .trigger:hover {
    border-color: var(--accent-soft);
    color: var(--fg-0);
  }
  .dot {
    display: inline-block;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid var(--border);
  }
  .label {
    font-weight: 500;
  }
  .menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 50;
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px;
    min-width: 140px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    color: var(--fg-1);
    text-align: left;
  }
  .item:hover {
    background: var(--bg-3);
  }
  .item.active {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .swatch {
    width: 14px;
    height: 14px;
    border-radius: 3px;
    border: 1px solid var(--border);
  }
  .name {
    flex: 1;
  }
  .check {
    color: var(--accent);
    font-size: 11px;
  }
</style>
