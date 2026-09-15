<script lang="ts">
  import { get } from "svelte/store";
  import { api } from "$lib/api";
  import {
    chatMessages,
    genParams,
    isGenerating,
    modelLoaded,
    modelStatus,
    notify,
    refreshModelStatus,
  } from "$lib/stores";
  import type { ChatMessage, StreamChunk } from "$lib/types";
  import { uuid } from "$lib/util";

  let input = $state("");
  let chatBody = $state<HTMLDivElement>();
  let unlisten: (() => void) | null = null;

  $effect(() => {
    void $chatMessages;
    void $isGenerating;
    if (chatBody) {
      requestAnimationFrame(() => {
        chatBody!.scrollTop = chatBody!.scrollHeight;
      });
    }
  });

  async function ensureListener() {
    if (unlisten) return;
    const { listen } = await import("@tauri-apps/api/event");
    unlisten = await listen<StreamChunk>("ai_chat_chunk", (e) => {
      const p = e.payload;
      if (p.requestId !== currentRequestId) return;
      const msgs = get(chatMessages);
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant") {
        const updated = { ...last, content: last.content + p.token };
        chatMessages.set([...msgs.slice(0, -1), updated]);
      } else if (p.token) {
        chatMessages.set([...msgs, { role: "assistant", content: p.token }]);
      }
      if (p.finish) {
        isGenerating.set(false);
        currentRequestId = "";
      }
    });
  }

  let currentRequestId = "";

  async function send() {
    if (!input.trim() || get(isGenerating)) return;
    if (!get(modelLoaded)) {
      notify("error", "Model not loaded. Check status in the AI panel header.");
      return;
    }
    const userMsg: ChatMessage = { role: "user", content: input.trim() };
    const messages = [...get(chatMessages), userMsg];
    chatMessages.set(messages);
    input = "";
    isGenerating.set(true);
    currentRequestId = uuid();

    await ensureListener();
    try {
      await api.aiChat(currentRequestId, messages, get(genParams));
    } catch (e) {
      notify("error", `AI chat failed: ${e}`);
      isGenerating.set(false);
      currentRequestId = "";
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  }

  function clearChat() {
    chatMessages.set([]);
  }

  async function reloadModel() {
    notify("info", "Loading model…");
    try {
      const s = await api.modelLoad();
      notify("success", s);
    } catch (e) {
      notify("error", `Load failed: ${e}`);
    }
    await refreshModelStatus();
  }
</script>

<div class="ai-panel">
  <div class="ai-header">
    <div class="ai-title">
      <span>AI</span>
      <span class="status" class:ok={$modelLoaded} class:bad={!$modelLoaded}>
        {$modelStatus}
      </span>
    </div>
    <div class="ai-actions">
      <button onclick={reloadModel} title="Reload model">↻</button>
      <button onclick={clearChat} title="Clear chat">⌫</button>
    </div>
  </div>

  <div class="ai-body" bind:this={chatBody}>
    {#if $chatMessages.length === 0}
      <div class="empty">
        <p>Ask the AI for help brainstorming, outlining, or revising your book.</p>
        <p class="hint">Try: <em>"Suggest a 5-chapter outline for a noir mystery set in 1970s Reykjavík."</em></p>
      </div>
    {:else}
      {#each $chatMessages as m (m)}
        <div class="msg {m.role}">
          <div class="role">{m.role}</div>
          <div class="content">{m.content || "…"}</div>
        </div>
      {/each}
      {#if $isGenerating}
        <div class="msg assistant">
          <div class="role">assistant</div>
          <div class="content typing">…</div>
        </div>
      {/if}
    {/if}
  </div>

  <div class="ai-input">
    <textarea
      bind:value={input}
      onkeydown={onKeydown}
      placeholder="Ask anything about your book… (Enter to send, Shift+Enter for newline)"
      rows="3"
      disabled={$isGenerating}
    ></textarea>
    <button class="primary" onclick={send} disabled={$isGenerating || !input.trim()}>
      {get(isGenerating) ? "Generating…" : "Send"}
    </button>
  </div>
</div>

<style>
  .ai-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-1);
  }
  .ai-header {
    height: 36px;
    padding: 0 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
  }
  .ai-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-2);
    font-weight: 600;
  }
  .status {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 3px;
    text-transform: none;
    letter-spacing: 0;
    font-family: var(--mono);
    background: var(--bg-3);
    color: var(--fg-1);
  }
  .status.ok {
    color: var(--success);
  }
  .status.bad {
    color: var(--warn);
  }
  .ai-actions {
    display: flex;
    gap: 4px;
  }
  .ai-actions button {
    padding: 2px 8px;
    font-size: 12px;
  }
  .ai-body {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .empty {
    color: var(--fg-2);
    font-size: 12px;
    text-align: center;
    padding: 24px 12px;
    line-height: 1.6;
  }
  .empty .hint {
    font-size: 11px;
    color: var(--fg-2);
    margin-top: 12px;
  }
  .msg {
    padding: 8px 10px;
    border-radius: 6px;
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  .msg.user {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .msg.assistant {
    background: var(--bg-2);
    color: var(--fg-0);
    border-left: 2px solid var(--accent);
  }
  .msg.system {
    background: transparent;
    color: var(--fg-2);
    font-style: italic;
    border: 1px dashed var(--border);
  }
  .role {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-2);
    margin-bottom: 4px;
    font-weight: 600;
  }
  .content.typing {
    color: var(--fg-2);
    font-style: italic;
  }
  .ai-input {
    padding: 8px;
    background: var(--bg-2);
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ai-input textarea {
    resize: none;
    font-size: 13px;
    line-height: 1.4;
    background: var(--bg-1);
    border-color: var(--border);
  }
  .ai-input button {
    align-self: flex-end;
  }
</style>
