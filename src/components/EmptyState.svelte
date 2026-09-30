<script lang="ts">
  interface Props {
    title: string;
    subtitle: string;
    cta: string;
    onAdd: () => void;
  }
  let { title, subtitle, cta, onAdd }: Props = $props();
</script>

<div class="empty-state fade-in">
  <div class="glow"></div>
  <div class="content">
    <div class="icon">✦</div>
    <h3>{title}</h3>
    <p>{subtitle}</p>
    <button class="primary" onclick={onAdd}>+ {cta}</button>
  </div>
</div>

<style>
  .empty-state {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 0;
    text-align: center;
  }
  .glow {
    position: absolute;
    width: 240px;
    height: 240px;
    border-radius: 50%;
    margin-bottom: -150px;
    background: radial-gradient(circle at center, var(--accent-soft) 0%, transparent 70%);
    opacity: 0.18;
  }
  .content {
    position: relative;
    z-index: 1;
  }
  .icon {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--paper);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    margin: 0 auto 16px;
  }
  h3 {
    margin: 0 0 8px 0;
    font-family: var(--serif);
    font-size: 20px;
    color: var(--fg-0);
  }
  p {
    margin: 0 0 20px 0;
    font-family: var(--serif);
    font-size: 14px;
    color: var(--fg-1);
    line-height: 1.6;
    max-width: 380px;
  }
  button.primary {
    font-size: 13px;
    padding: 8px 16px;
  }
</style>
