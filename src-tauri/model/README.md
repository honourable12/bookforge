# Model files

BookForge looks for GGUF model files in this directory at startup.

## Default model

Run the downloader from the project root:

```powershell
.\scripts\download_model.ps1
```

This fetches `qwen2.5-1.5b-instruct-q4_k_m.gguf` (~1 GB) from HuggingFace
and places it in this directory.

## Custom model

Place any GGUF file in this directory, or set the `BOOKFORGE_MODEL`
environment variable to an absolute path:

```powershell
$env:BOOKFORGE_MODEL = "C:\path\to\your.gguf"
```

Supported model families include Qwen2.5, Llama-3, Mistral, Phi-3, Gemma,
and any other model that produces a GGUF file.

## Editing the default filename

To use a different default filename (so you don't need to set the env var
every time), edit `TARGET_FILENAME` in `src-tauri\src\model.rs`:

```rust
pub const TARGET_FILENAME: &str = "my-custom-model.gguf";
```

Then rebuild.
