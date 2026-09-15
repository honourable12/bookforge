<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import "@xterm/xterm/css/xterm.css";
  import { api } from "$lib/api";
  import { project, notify } from "$lib/stores";

  interface Props {
    sessionId?: string | null;
    title?: string;
  }
  let { sessionId = $bindable(null), title = $bindable("Terminal") }: Props = $props();

  let host = $state<HTMLDivElement>();
  let term: Terminal | null = null;
  let fit: FitAddon | null = null;
  let unlisten: (() => void) | null = null;

  async function init() {
    if (!host) return;
    term = new Terminal({
      fontFamily: "var(--mono)",
      fontSize: 13,
      cursorBlink: true,
      theme: {
        background: "#0f1115",
        foreground: "#e6e9ef",
        cursor: "#7c9cff",
        selectionBackground: "rgba(124,156,255,0.3)",
        black: "#0f1115",
        red: "#f87171",
        green: "#6ee7b7",
        yellow: "#fbbf24",
        blue: "#7c9cff",
        magenta: "#b48cff",
        cyan: "#67e8f9",
        white: "#e6e9ef",
        brightBlack: "#6b7280",
        brightRed: "#fca5a5",
        brightGreen: "#86efac",
        brightYellow: "#fde68a",
        brightBlue: "#93c5fd",
        brightMagenta: "#c4b5fd",
        brightCyan: "#a5f3fc",
        brightWhite: "#f3f4f6",
      },
    });
    fit = new FitAddon();
    term.loadAddon(fit);
    term.loadAddon(new WebLinksAddon());
    term.open(host);
    fit.fit();

    const cwd = $project?.root ?? undefined;
    try {
      sessionId = await api.termSpawn({
        cwd,
        cols: term.cols,
        rows: term.rows,
      });
      title = `Terminal ${sessionId.slice(0, 8)}`;
    } catch (e) {
      notify("error", `Failed to spawn terminal: ${e}`);
      return;
    }

    const { listen } = await import("@tauri-apps/api/event");
    unlisten = await listen<{ sessionId: string; data: string }>(
      "term_output",
      (e) => {
        if (e.payload.sessionId !== sessionId) return;
        term?.write(e.payload.data);
      },
    );

    term.onData((data) => {
      if (sessionId) api.termWrite(sessionId, data).catch(() => {});
    });

    term.onResize(({ cols, rows }) => {
      if (sessionId) api.termResize(sessionId, cols, rows).catch(() => {});
    });

    const onResize = () => fit?.fit();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }

  export async function kill() {
    if (sessionId) {
      try {
        await api.termKill(sessionId);
      } catch (e) {
        notify("error", `Failed to kill terminal: ${e}`);
      }
    }
  }

  onMount(() => {
    void init();
  });
  onDestroy(() => {
    if (unlisten) unlisten();
    if (term) term.dispose();
    if (sessionId) api.termKill(sessionId).catch(() => {});
  });
</script>

<div class="terminal-pane">
  <div class="terminal-titlebar">
    <span class="dot"></span>
    <span class="title">{title}</span>
  </div>
  <div class="terminal-body" bind:this={host}></div>
</div>

<style>
  .terminal-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-1);
  }
  .terminal-titlebar {
    height: 28px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
    color: var(--fg-1);
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success);
  }
  .title {
    flex: 1;
    font-family: var(--mono);
  }
  .terminal-body {
    flex: 1;
    overflow: hidden;
    padding: 4px 0 0 4px;
  }
</style>
