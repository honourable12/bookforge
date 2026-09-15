<script lang="ts">
  import { get } from "svelte/store";
  import { marked } from "marked";
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

  interface Props {
    onInsertText?: (text: string) => void;
    onAppendText?: (text: string) => void;
    onReplaceText?: (text: string) => void;
  }

  let { onInsertText, onAppendText, onReplaceText }: Props = $props();

  let input = $state("");
  let chatBody = $state<HTMLDivElement>();
  let unlisten: (() => void) | null = null;
  let targetMode = $state<"chat" | "editor">("chat");
  let currentRequestId = "";
  let copiedIndex = $state<number | null>(null);
  let copyTimeout: any = null;

  let lastMessage = $derived($chatMessages[$chatMessages.length - 1]);

  $effect(() => {
    void $chatMessages;
    void $isGenerating;
    if (chatBody) {
      requestAnimationFrame(() => {
        chatBody!.scrollTop = chatBody!.scrollHeight;
      });
    }
  });

  function renderMarkdown(raw: string): string {
    if (!raw) return "";
    try {
      return marked.parse(raw, { gfm: true, breaks: true }) as string;
    } catch (err) {
      console.error("Markdown parse error:", err);
      return raw;
    }
  }

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

      if (targetMode === "editor" && p.token) {
        onInsertText?.(p.token);
      }

      if (p.finish) {
        isGenerating.set(false);
        currentRequestId = "";
        targetMode = "chat";
      }
    });
  }

  async function send(mode: "chat" | "editor" = "chat") {
    if (!input.trim() || get(isGenerating)) return;
    if (!get(modelLoaded)) {
      notify("error", "Model not loaded. Check status in the AI panel header.");
      return;
    }
    if (mode === "editor" && !onInsertText) {
      notify("info", "Open a chapter in the editor first to write into.");
      return;
    }

    targetMode = mode;
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
      targetMode = "chat";
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      send("editor");
    } else if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      send("chat");
    }
  }

  function handleInsert(text: string) {
    if (!onInsertText) {
      notify("info", "No chapter open in editor.");
      return;
    }
    onInsertText(text);
    notify("success", "Inserted at cursor in open chapter.");
  }

  function handleAppend(text: string) {
    if (!onAppendText) {
      notify("info", "No chapter open in editor.");
      return;
    }
    onAppendText(text);
    notify("success", "Appended to end of open chapter.");
  }

  function handleReplace(text: string) {
    if (!onReplaceText) {
      notify("info", "No chapter open in editor.");
      return;
    }
    onReplaceText(text);
    notify("success", "Replaced selected text in chapter.");
  }

  async function handleCopy(text: string, index: number) {
    try {
      await navigator.clipboard.writeText(text);
      copiedIndex = index;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copiedIndex = null;
      }, 2000);
    } catch {
      notify("error", "Failed to copy to clipboard.");
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
      <span>AI Assistant</span>
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
    {#if $chatMessages.length === 0 && !$isGenerating}
      <div class="empty">
        <p>Brainstorm scenes, draft chapters, or have the AI write directly into your book.</p>
        <p class="hint">Try: <em>"Suggest a 5-chapter outline for a mystery set in 1970s Reykjavík."</em></p>
        <div class="shortcuts-guide">
          <div><kbd>Enter</kbd> Send to Chat</div>
          <div><kbd>Ctrl</kbd>+<kbd>Enter</kbd> Write to Book</div>
        </div>
      </div>
    {:else}
      {#each $chatMessages as m, i (i)}
        <div class="msg {m.role}">
          <div class="msg-header">
            <span class="role">{m.role}</span>
            {#if m.role === "assistant" && (!$isGenerating || i < $chatMessages.length - 1)}
              <div class="msg-actions">
                <button
                  type="button"
                  class="action-pill"
                  onclick={() => handleInsert(m.content)}
                  title="Insert at cursor in open chapter"
                >
                  📥 Insert
                </button>
                <button
                  type="button"
                  class="action-pill"
                  onclick={() => handleAppend(m.content)}
                  title="Append to end of open chapter"
                >
                  ➕ Append
                </button>
                <button
                  type="button"
                  class="action-pill"
                  onclick={() => handleReplace(m.content)}
                  title="Replace selected text in editor"
                >
                  ✎ Replace
                </button>
                <button
                  type="button"
                  class="action-pill"
                  onclick={() => handleCopy(m.content, i)}
                  title="Copy to clipboard"
                >
                  {copiedIndex === i ? "✓ Copied" : "📋 Copy"}
                </button>
              </div>
            {/if}
          </div>

          <div class="content">
            {#if m.role === "assistant"}
              <div class="prose">
                {@html renderMarkdown(m.content)}
                {#if $isGenerating && i === $chatMessages.length - 1}
                  <span class="streaming-cursor"></span>
                {/if}
              </div>
            {:else}
              <div class="user-text">{m.content}</div>
            {/if}
          </div>
        </div>
      {/each}

      {#if $isGenerating && (!lastMessage || lastMessage.role !== "assistant" || !lastMessage.content)}
        <div class="msg assistant thinking-card">
          <div class="msg-header">
            <span class="role">assistant</span>
          </div>
          <div class="content thinking">
            <span class="dot"></span>
            <span class="dot"></span>
            <span class="dot"></span>
            <span class="thinking-text">{targetMode === "editor" ? "Writing to book…" : "Thinking…"}</span>
          </div>
        </div>
      {/if}
    {/if}
  </div>

  <div class="ai-input">
    <textarea
      bind:value={input}
      onkeydown={onKeydown}
      placeholder="Ask anything or request a scene… (Enter = Chat, Ctrl+Enter = Write to Book)"
      rows="3"
      disabled={$isGenerating}
    ></textarea>
    <div class="input-toolbar">
      <div class="input-hint">
        {#if targetMode === "editor" && $isGenerating}
          <span class="writing-indicator">✦ Writing directly into chapter…</span>
        {/if}
      </div>
      <div class="input-buttons">
        <button
          type="button"
          class="write-btn"
          onclick={() => send("editor")}
          disabled={$isGenerating || !input.trim()}
          title="Streams generated prose directly into the open chapter at the cursor (Ctrl+Enter)"
        >
          ✦ Write to Book
        </button>
        <button
          type="button"
          class="primary"
          onclick={() => send("chat")}
          disabled={$isGenerating || !input.trim()}
          title="Send to AI chat panel (Enter)"
        >
          {$isGenerating && targetMode === "chat" ? "Generating…" : "Send"}
        </button>
      </div>
    </div>
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
    gap: 12px;
    -webkit-user-select: text;
    user-select: text;
  }
  .empty {
    color: var(--fg-2);
    font-size: 12px;
    text-align: center;
    padding: 28px 12px;
    line-height: 1.6;
  }
  .empty .hint {
    font-size: 11px;
    color: var(--fg-2);
    margin-top: 10px;
  }
  .shortcuts-guide {
    margin-top: 18px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    font-size: 11px;
    color: var(--fg-1);
  }
  .shortcuts-guide kbd {
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 5px;
    font-family: var(--mono);
    font-size: 10px;
    color: var(--fg-0);
  }
  .msg {
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 13px;
    line-height: 1.5;
    word-wrap: break-word;
    overflow-wrap: break-word;
  }
  .msg.user {
    background: var(--bg-3);
    color: var(--fg-0);
  }
  .msg.assistant {
    background: var(--bg-2);
    color: var(--fg-0);
    border-left: 3px solid var(--accent);
  }
  .msg-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
    flex-wrap: wrap;
    gap: 4px;
  }
  .role {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--fg-2);
    font-weight: 600;
  }
  .msg-actions {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .action-pill {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--bg-3);
    color: var(--fg-1);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    line-height: 1.2;
    transition: all 0.15s ease;
  }
  .action-pill:hover {
    background: var(--border);
    color: var(--fg-0);
    border-color: var(--accent);
  }
  .user-text {
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.5;
  }

  /* Rich prose rendering for generated text */
  .prose {
    line-height: 1.6;
    font-size: 13px;
    color: var(--fg-0);
  }
  .prose :global(p) {
    margin: 0 0 8px 0;
  }
  .prose :global(p:last-child) {
    margin-bottom: 0;
  }
  .prose :global(h1),
  .prose :global(h2),
  .prose :global(h3),
  .prose :global(h4) {
    color: var(--fg-0);
    margin: 14px 0 6px 0;
    font-weight: 600;
    line-height: 1.3;
  }
  .prose :global(h1:first-child),
  .prose :global(h2:first-child),
  .prose :global(h3:first-child) {
    margin-top: 0;
  }
  .prose :global(h1) { font-size: 16px; border-bottom: 1px solid var(--border); padding-bottom: 4px; }
  .prose :global(h2) { font-size: 14px; }
  .prose :global(h3) { font-size: 13px; }
  .prose :global(ul),
  .prose :global(ol) {
    margin: 4px 0 8px 0;
    padding-left: 20px;
  }
  .prose :global(li) {
    margin-bottom: 4px;
  }
  .prose :global(blockquote) {
    margin: 8px 0;
    padding: 4px 10px;
    border-left: 3px solid var(--accent-2);
    color: var(--fg-1);
    background: var(--bg-1);
    border-radius: 0 4px 4px 0;
    font-style: italic;
  }
  .prose :global(pre) {
    background: var(--bg-0);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px;
    overflow-x: auto;
    font-family: var(--mono);
    font-size: 12px;
    line-height: 1.45;
    margin: 8px 0;
  }
  .prose :global(code) {
    font-family: var(--mono);
    font-size: 11px;
    background: var(--bg-3);
    padding: 1px 4px;
    border-radius: 4px;
    color: var(--accent);
  }
  .prose :global(pre code) {
    background: transparent;
    padding: 0;
    color: var(--fg-0);
  }
  .prose :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 12px 0;
  }
  .prose :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 8px 0;
    font-size: 12px;
  }
  .prose :global(th),
  .prose :global(td) {
    border: 1px solid var(--border);
    padding: 4px 8px;
    text-align: left;
  }
  .prose :global(th) {
    background: var(--bg-3);
    color: var(--fg-0);
  }

  /* Streaming cursor */
  .streaming-cursor {
    display: inline-block;
    width: 6px;
    height: 14px;
    vertical-align: -2px;
    margin-left: 2px;
    background-color: var(--accent);
    animation: blink 0.8s infinite;
  }
  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }

  /* Thinking animation */
  .thinking-card {
    border-left-style: dashed;
  }
  .thinking {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 0;
    color: var(--fg-2);
    font-size: 12px;
  }
  .dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background-color: var(--accent);
    animation: bounce 1.4s infinite ease-in-out both;
  }
  .dot:nth-child(1) { animation-delay: -0.32s; }
  .dot:nth-child(2) { animation-delay: -0.16s; }
  @keyframes bounce {
    0%, 80%, 100% { transform: scale(0); }
    40% { transform: scale(1); }
  }
  .thinking-text {
    margin-left: 4px;
    font-style: italic;
  }

  /* Input area */
  .ai-input {
    padding: 8px 10px;
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
  .input-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }
  .input-hint {
    font-size: 11px;
    color: var(--accent-2);
    font-weight: 500;
  }
  .writing-indicator {
    display: flex;
    align-items: center;
    gap: 4px;
    animation: pulse 1.5s infinite ease-in-out;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  .input-buttons {
    display: flex;
    gap: 6px;
  }
  .write-btn {
    background: var(--bg-3);
    border: 1px solid var(--accent-2);
    color: var(--accent-2);
    font-size: 12px;
    font-weight: 500;
    padding: 5px 10px;
    transition: all 0.15s ease;
  }
  .write-btn:hover:not(:disabled) {
    background: var(--accent-2);
    color: #0f1115;
  }
  .ai-input button.primary {
    font-size: 12px;
    padding: 5px 12px;
  }
</style>
