# BookForge

A **Tauri-based book-writing IDE** for Windows with an embedded **local AI assistant** (via `llama.cpp`), a real **terminal/REPL**, and one-click **HTML / PDF / EPUB** export.

Built with:
- **Rust + Tauri 2** for the native shell and AI inference
- **Svelte 4 + Vite + TypeScript** for the UI
- **CodeMirror 6** for the prose editor
- **xterm.js** for the embedded terminal
- **`llama-cpp-2`** for in-process LLM inference — no Python, no server, no API keys

The default model is `qwen2.5-1.5b-instruct-q4_k_m.gguf` (~1 GB), which fits comfortably in RAM on a laptop and loads in a few seconds. Any GGUF model is supported via the `BOOKFORGE_MODEL` env var.

---

## Features

- **Directory-based projects** — each project is a folder with `bookforge.json`, `chapters\*.md`, `assets\`, and `exports\`. Sync via git, Dropbox, OneDrive, etc.
- **Markdown editor** with auto-save (1.5s debounce) and word counts per chapter.
- **Embedded AI panel** — chat with the model about your book, ask for outlines, character bios, etc.
- **AI continue** — click "Continue" in the toolbar and the model writes a continuation of the paragraph before your cursor, streamed live into the document.
- **AI rewrite** — select a passage, type an instruction ("make it more lyrical", "switch to past tense"), and the model replaces the selection.
- **Embedded terminal/REPL** — a real PTY-backed shell running inside the app. On Windows it defaults to **PowerShell** (PowerShell Core if installed, else Windows PowerShell 5.1, else `cmd.exe`). Use it for `git`, `python`, `node`, `pwsh`, or any REPL.
- **Export to HTML / PDF / EPUB** — one click each. Output goes to `<project>\exports\`.
- **No network required** — model inference is 100% local. The only network call is the initial model download.

---

## Quickstart (Windows)

### Prerequisites

1. **Rust** (stable, 1.77+) — <https://rustup.rs/>
   - During install, accept the default MSVC toolchain.
2. **Node.js** 18+ and **npm** — <https://nodejs.org/>
3. **Microsoft Visual C++ Build Tools** — <https://visualstudio.microsoft.com/visual-cpp-build-tools/>
   - Select the **"Desktop development with C++"** workload.
   - Required because `llama-cpp-2` compiles `llama.cpp` from source via CMake + MSVC.
4. **CMake** — <https://cmake.org/download/>
   - Needed by `llama-cpp-2`'s build script.
5. **PowerShell 7+** (optional but recommended) — <https://github.com/PowerShell/PowerShell>
   - Needed for the setup/download scripts. Windows PowerShell 5.1 also works.

### One-shot setup

Open **PowerShell** (as a regular user — no admin needed) in the project root and run:

```powershell
.\scripts\setup-windows.ps1
```

This will:
1. Verify Rust, Node.js, npm, and MSVC are installed
2. Run `npm install` for the frontend
3. Download the default GGUF model into `src-tauri\model\`

If anything is missing, the script will tell you what to install and exit.

### Manual setup (if you prefer)

```powershell
# 1. Install frontend deps
npm install

# 2. Download the default GGUF model (~1 GB, one-time)
.\scripts\download_model.ps1

# 3. Launch in dev mode (Rust + Svelte hot reload)
npm run tauri:dev
```

For a production build (produces a Windows installer):

```powershell
npm run tauri:build
# Output:
#   src-tauri\target\release\bundle\msi\BookForge_0.1.0_x64_en-US.msi
#   src-tauri\target\release\bundle\nsis\BookForge_0.1.0_x64-setup.exe
```

The MSI and NSIS installers both bundle the model file (from `src-tauri\model\`) into the application resources directory.

### Using a different model

Set the `BOOKFORGE_MODEL` env var to an absolute path before launching. Any GGUF model works — `llama-cpp-2` supports Llama, Qwen, Mistral, Phi, Gemma, etc.

```powershell
$env:BOOKFORGE_MODEL = "C:\models\llama-3-8b-instruct-q4_k_m.gguf"
npm run tauri:dev
```

Or set it permanently via the Windows settings (System → About → Advanced system settings → Environment Variables).

You can also drop a model file at any of these paths and BookForge will pick it up automatically:
- `<cwd>\model\<filename>.gguf`
- `<cwd>\..\model\<filename>.gguf`
- `<exe_dir>\model\<filename>.gguf`

The expected filename is `qwen2.5-1.5b-instruct-q4_k_m.gguf` (override by editing `src-tauri\src\model.rs::TARGET_FILENAME`).

---

## Project layout on disk

A BookForge project is just a folder:

```
my-book\
├── bookforge.json        # metadata: title, author, language, version
├── chapters\
│   ├── 01-introduction.md
│   ├── 02-chapter-one.md
│   └── 99-epilogue.md
├── assets\               # images, cover, etc.
└── exports\              # written by the export commands
    ├── my-book.html
    ├── my-book.pdf
    └── my-book.epub
```

Chapter filenames that start with a number (like `01-`, `02-`) are sorted numerically; otherwise alphabetical. The first `# Heading` in a chapter file becomes the chapter title shown in the sidebar.

The `bookforge.json` is plain JSON and fully editable by hand:

```json
{
  "title": "My Cool Book",
  "author": "Jane Q. Author",
  "language": "en",
  "description": "",
  "version": "0.1.0",
  "createdAt": "2025-01-01T00:00:00Z",
  "updatedAt": "2025-01-02T00:00:00Z"
}
```

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                         Svelte frontend                          │
│  ┌───────────┐ ┌────────────────────────┐ ┌──────────────────┐   │
│  │  Sidebar  │ │  CodeMirror editor     │ │   AI chat panel  │   │
│  │  project  │ │  (markdown, autosave)  │ │   (chat stream)  │   │
│  │  tree     │ │                        │ │                  │   │
│  │  + meta   │ ├────────────────────────┤ ├──────────────────┤   │
│  │           │ │  xterm.js terminal     │ │  AI continue /   │   │
│  │           │ │  (PTY-backed)          │ │  rewrite toolbar │   │
│  └───────────┘ └────────────────────────┘ └──────────────────┘   │
│                          status bar                              │
└──────────────────────────────┬───────────────────────────────────┘
                               │ Tauri IPC (invoke + events)
┌──────────────────────────────┴───────────────────────────────────┐
│                            Rust backend                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │  model   │ │   ai     │ │ project  │ │  export  │ │ terminal│ │
│  │  loader  │ │ inference│ │  CRUD    │ │ html/pdf │ │  PTY    │ │
│  │ (OnceLock│ │ streamer │ │  (fs)    │ │  /epub   │ │         │ │
│  │  static) │ │          │ │          │ │          │ │         │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └─────────┘ │
│       │                                                              │
│       └── llama-cpp-2 → llama.cpp (statically linked, MSVC)         │
└──────────────────────────────────────────────────────────────────┘
```

### Modules

| File | Responsibility |
|---|---|
| `src-tauri\src\model.rs` | Backend + model singletons, path lookup, context params. Adapted from the reference snippet. |
| `src-tauri\src\ai.rs` | Tokenized prompt building (Qwen2.5 chat template), `run_completion()` inference loop, Tauri event streaming. |
| `src-tauri\src\project.rs` | Directory-based project CRUD: `bookforge.json`, chapters, assets. |
| `src-tauri\src\export.rs` | HTML (pulldown-cmark), PDF (printpdf), EPUB (epub-builder). |
| `src-tauri\src\terminal.rs` | PTY session registry, spawn/write/resize/kill via `portable-pty`. Windows default shell: PowerShell. |
| `src-tauri\src\commands.rs` | `#[tauri::command]` wrappers exposed to the frontend. |
| `src-tauri\src\lib.rs` | Tauri builder + plugin wiring + setup hook. |
| `src\lib\api.ts` | Typed `invoke()` wrappers for every command. |
| `src\lib\stores.ts` | Svelte stores for project state, chat, model status, terminals. |
| `src\components\Editor.svelte` | CodeMirror 6 + AI continue/rewrite helpers. |
| `src\components\Terminal.svelte` | xterm.js + Tauri event listener for PTY output. |
| `src\components\AIPanel.svelte` | Chat UI with streaming assistant messages. |
| `src\components\Sidebar.svelte` | Project tree + chapter list + new-project dialog. |
| `src\components\Toolbar.svelte` | New chapter, continue, rewrite, export buttons. |
| `src\components\StatusBar.svelte` | Project / chapter / model status + toasts. |
| `scripts\setup-windows.ps1` | One-shot Windows setup (verifies toolchain, installs npm deps, downloads model). |
| `scripts\download_model.ps1` | PowerShell downloader for the default GGUF model. |

---

## IPC reference

Every command listed in `src\lib\api.ts` is a `#[tauri::command]` in `src-tauri\src\commands.rs`. The streaming commands (`ai_chat`, `ai_continue`, `ai_rewrite`) emit Tauri events:

| Event | Payload | Fired by |
|---|---|---|
| `ai_chat_chunk` | `{ requestId, token, finish, error }` | `ai::run_completion` during chat |
| `ai_continue_chunk` | same | during "Continue writing" |
| `ai_rewrite_chunk` | same | during "Rewrite selection" |
| `term_output` | `{ sessionId, data }` | PTY reader thread |

The frontend listens via `@tauri-apps/api/event`'s `listen()` and routes by `requestId` / `sessionId`.

---

## Configuration

### Sampling defaults

`src\lib\stores.ts::DEFAULT_GEN_PARAMS`:

```ts
{
  maxTokens: 512,
  temperature: 0.7,
  topP: 0.9,
  topK: 40,
  repeatPenalty: 1.1,
  seed: 0xcafebabe
}
```

The Rust `GenParams` struct mirrors this in `src-tauri\src\ai.rs`. The current sampler is a simple argmax-after-temperature; full top-k / top-p / repetition-penalty sampling is left as a TODO (the parameters are wired through but not yet applied).

### Context size

`src-tauri\src\model.rs::N_CTX = 3072`. Increase if you have RAM to spare and want longer prompts (e.g. when continuing from a 50K-word chapter prefix).

### GPU acceleration

The model is loaded with `with_n_gpu_layers(0)` (CPU-only) for portability. On a machine with a CUDA-capable NVIDIA GPU, you can enable CUDA offload:

1. Install the CUDA Toolkit 12+ (<https://developer.nvidia.com/cuda-toolkit>)
2. Set `CMAKE_ARGS="-DLLAMA_CUDA=on"` before `npm run tauri:dev`:
   ```powershell
   $env:CMAKE_ARGS = "-DLLAMA_CUDA=on"
   npm run tauri:dev
   ```
3. Change in `src-tauri\src\model.rs`:
   ```rust
   let model_params = LlamaModelParams::default()
       .with_n_gpu_layers(1000)  // offload all layers
       .with_use_mmap(true);
   ```

For Vulkan (works with AMD/Intel/NVIDIA GPUs): `-DLLAMA_VULKAN=on`.

---

## Export quality notes

- **HTML**: full CSS with print-friendly media queries. Opens in any browser; use the browser's "Print to PDF" for a high-fidelity PDF if the built-in PDF exporter isn't sufficient.
- **PDF**: simple typeset via `printpdf`. Renders paragraphs with word wrap and per-chapter page breaks. Good enough for proofing; for final-quality PDFs use the HTML export + browser print.
- **EPUB**: uses `epub-builder`. Generates a valid EPUB 2 with inline CSS and per-chapter XHTML. Reads cleanly in Apple Books, Calibre, Kindle (via conversion).

---

## Troubleshooting (Windows)

**"Model file not found"** — Run `.\scripts\download_model.ps1` from the project root. Or set `$env:BOOKFORGE_MODEL = "C:\path\to\your.gguf"`.

**`cargo build` fails with "link.exe not found"** — You're missing the MSVC Build Tools. Install them from <https://visualstudio.microsoft.com/visual-cpp-build-tools/> (select "Desktop development with C++").

**`cargo build` fails inside `llama-cpp-2`** — Make sure `cmake` is installed and on PATH: `cmake --version`. If not, install from <https://cmake.org/download/>.

**Terminal shows `cmd.exe` instead of PowerShell** — BookForge looks for `pwsh.exe` (PowerShell Core) on PATH first, then falls back to `powershell.exe` in the well-known System32 location. If neither is found it falls back to `cmd.exe`. Install PowerShell 7+ from <https://github.com/PowerShell/PowerShell> for the best experience.

**AI output is gibberish** — Make sure you're using an **instruct** model (e.g. `qwen2.5-1.5b-instruct-q4_k_m.gguf`, not the base model). The chat template in `ai.rs` is hard-coded to the Qwen2.5 format (`<|im_start|>...`). For other model families (Llama-3, Mistral), update `build_chat_prompt()` to match.

**Antivirus blocks the installer** — The bundled model file is large and unsigned. You may need to allow it through Windows Defender or your AV product.

**Tauri dev server fails to start** — Make sure port 1420 is free. If something is using it, kill that process or change the port in `vite.config.ts` and `tauri.conf.json`.

---

## License

MIT. See `LICENSE`.
