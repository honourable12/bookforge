<script lang="ts">
  interface Props {
    value: number;
    max: number;
    size?: number;
    stroke?: number;
    color?: string;
    trackColor?: string;
    showLabel?: boolean;
    label?: string;
  }

  let {
    value,
    max,
    size = 56,
    stroke = 4,
    color = "var(--accent)",
    trackColor = "var(--border-soft)",
    showLabel = true,
    label,
  }: Props = $props();

  let radius = $derived((size - stroke) / 2);
  let circumference = $derived(2 * Math.PI * radius);
  let pct = $derived(Math.max(0, Math.min(1, max > 0 ? value / max : 0)));
  let offset = $derived(circumference * (1 - pct));
  let labelText = $derived(label ?? Math.round(pct * 100) + "%");
</script>

<div class="ring" style="width: {size}px; height: {size}px">
  <svg width={size} height={size} class="svg">
    <circle cx={size / 2} cy={size / 2} {radius} fill="none" stroke={trackColor} stroke-width={stroke} />
    <circle
      class="progress-ring-circle"
      cx={size / 2}
      cy={size / 2}
      {radius}
      fill="none"
      stroke={color}
      stroke-width={stroke}
      stroke-dasharray={circumference}
      stroke-dashoffset={offset}
      stroke-linecap="round"
    />
  </svg>
  {#if showLabel}
    <div class="label">
      <span style="font-size: {size * 0.22}px">{labelText}</span>
    </div>
  {/if}
</div>

<style>
  .ring {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .svg {
    transform: rotate(-90deg);
  }
  .label {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--serif);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    color: var(--fg-0);
  }
</style>
