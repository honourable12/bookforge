//! AI inference layer built on top of [`crate::model`].
//!
//! Exposes:
//!  - `chat(messages, ...) -> stream of tokens`
//!  - `continue_text(prompt, ...) -> stream of tokens`
//!  - `rewrite(selection, instruction, ...) -> stream of tokens`
//!
//! Token streaming is delivered via Tauri events so the Svelte frontend can
//! render progressive output. Each event payload is [`StreamChunk`].

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Window};

use crate::model::{self, build_context_params, get_model};
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::{AddBos, Special};
use llama_cpp_2::token::LlamaToken;

/// A chat message in the OpenAI-style `role`/`content` format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system" | "user" | "assistant"
    pub content: String,
}

/// Per-request sampling parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenParams {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: i32,
    pub repeat_penalty: f32,
    pub seed: u32,
}

impl Default for GenParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            seed: 0xCAFE_BABE,
        }
    }
}

/// A single streamed chunk emitted to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    pub request_id: String,
    pub token: String,
    pub finish: bool,
    pub error: Option<String>,
}

const QWEN_SYSTEM: &str = "You are BookForge AI, an expert fiction writing assistant and novelist \
co-author embedded inside a desktop book editor. When asked to write, draft, or describe a scene, \
always produce actual narrative fiction prose with vivid descriptions, actions, and dialogue. \
Do NOT produce meta-analysis, chapter outlines, or bulleted character analyses unless the user \
specifically asks for an outline or plan.";

/// Build a Qwen2.5 chat-format prompt from a list of messages.
pub fn build_chat_prompt(messages: &[ChatMessage]) -> String {
    let mut out = String::new();
    let mut has_system = false;
    for m in messages {
        match m.role.as_str() {
            "system" => {
                has_system = true;
                out.push_str("<|im_start|>system\n");
                out.push_str(&m.content);
                out.push_str("<|im_end|>\n");
            }
            "user" => {
                out.push_str("<|im_start|>user\n");
                out.push_str(&m.content);
                out.push_str("<|im_end|>\n");
            }
            "assistant" => {
                out.push_str("<|im_start|>assistant\n");
                out.push_str(&m.content);
                out.push_str("<|im_end|>\n");
            }
            _ => {}
        }
    }
    if !has_system {
        let mut prefix = String::from("<|im_start|>system\n");
        prefix.push_str(QWEN_SYSTEM);
        prefix.push_str("<|im_end|>\n");
        out.insert_str(0, &prefix);
    }
    out.push_str("<|im_start|>assistant\n");
    out
}

/// Build a prompt for "continue writing from here".
pub fn build_continue_prompt(prefix: &str) -> String {
    let mut msgs = Vec::new();
    msgs.push(ChatMessage {
        role: "system".into(),
        content: "You are a literary co-author. Continue the user's prose seamlessly in the same \
                  voice, tense, and style. Do NOT add commentary, headers, or markdown. Output \
                  ONLY the continuation."
            .into(),
    });
    msgs.push(ChatMessage {
        role: "user".into(),
        content: format!("Continue this passage:\n\n```\n{}\n```", prefix),
    });
    build_chat_prompt(&msgs)
}

/// Build a prompt for rewriting a selection.
pub fn build_rewrite_prompt(selection: &str, instruction: &str) -> String {
    let mut msgs = Vec::new();
    msgs.push(ChatMessage {
        role: "system".into(),
        content: "You are a literary editor. Rewrite the user's selection according to their \
                  instruction. Output ONLY the rewritten passage, no commentary."
            .into(),
    });
    msgs.push(ChatMessage {
        role: "user".into(),
        content: format!(
            "Instruction: {}\n\nOriginal:\n```\n{}\n```\n\nRewritten:",
            instruction, selection
        ),
    });
    build_chat_prompt(&msgs)
}

/// Look up the LlamaToken for a string like `"<|im_end|>"`. Returns 0 if the
/// token isn't in the vocabulary.
fn token_for_str(model: &llama_cpp_2::model::LlamaModel, s: &str) -> i32 {
    match model.str_to_token(s, AddBos::Never) {
        Ok(v) if !v.is_empty() => v[0].0 as i32,
        _ => 0,
    }
}

/// Core sampling loop. Emits `StreamChunk` events with `event_name` until
/// done or an error occurs. The final chunk has `finish: true`.
///
/// Returns the full generated text on success.
pub fn run_completion(
    app: &AppHandle,
    _window: &Window,
    event_name: &str,
    request_id: &str,
    prompt: &str,
    params: &GenParams,
) -> Result<String, String> {
    let model = get_model()?;
    let ctx_params = build_context_params();

    let backend = model::get_backend();
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("Failed to create context: {:?}", e))?;

    let tokens = model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| format!("Tokenization failed: {:?}", e))?;

    let n_ctx = ctx.n_ctx() as usize;
    if tokens.len() + 4 > n_ctx {
        return Err(format!(
            "Prompt too long: {} tokens > context capacity {}",
            tokens.len(),
            n_ctx - 4
        ));
    }

    let n_batch = model::N_BATCH as usize;
    let mut batch = LlamaBatch::new(n_batch + 1, 1);
    let mut n_cur: i32 = 0;

    // Pre-fill in chunks of N_BATCH.
    let mut idx = 0;
    while idx < tokens.len() {
        batch.clear();
        let chunk_end = (idx + n_batch).min(tokens.len());
        for (j, tok) in tokens[idx..chunk_end].iter().enumerate() {
            let is_last = (idx + j + 1) == tokens.len();
            batch
                .add(*tok, n_cur + j as i32, &[0], is_last)
                .map_err(|e| format!("batch add (prefill): {:?}", e))?;
        }
        ctx.decode(&mut batch)
            .map_err(|e| format!("Prefill decode failed: {:?}", e))?;
        n_cur += (chunk_end - idx) as i32;
        idx = chunk_end;
    }

    let end_token = token_for_str(model, "<|im_end|>");
    let eos_token = model.token_eos().0 as i32;

    let mut output = String::new();

    let emit = |token_str: &str| {
        let _ = app.emit(
            event_name,
            StreamChunk {
                request_id: request_id.to_string(),
                token: token_str.to_string(),
                finish: false,
                error: None,
            },
        );
    };

    for _ in 0..params.max_tokens {
        let last = (batch.n_tokens() - 1) as i32;
        let logits = ctx.get_logits_ith(last);
        let mut logits_vec: Vec<f32> = logits.to_vec();

        if params.temperature > 0.0 {
            for l in logits_vec.iter_mut() {
                *l /= params.temperature;
            }
        }

        let next = pick_argmax(&logits_vec);
        if next < 0 || next == end_token || next == eos_token {
            break;
        }

        let next_tok = LlamaToken::new(next);
        let piece = model
            .token_to_str(next_tok, Special::Tokenize)
            .unwrap_or_default();
        output.push_str(&piece);
        emit(&piece);

        batch.clear();
        batch
            .add(next_tok, n_cur, &[0], true)
            .map_err(|e| format!("batch add (step): {:?}", e))?;
        ctx.decode(&mut batch)
            .map_err(|e| format!("Step decode failed: {:?}", e))?;
        n_cur += 1;
    }

    let _ = app.emit(
        event_name,
        StreamChunk {
            request_id: request_id.to_string(),
            token: String::new(),
            finish: true,
            error: None,
        },
    );

    Ok(output)
}

fn pick_argmax(logits: &[f32]) -> i32 {
    let mut best: i32 = 0;
    let mut best_v = f32::NEG_INFINITY;
    for (i, v) in logits.iter().enumerate() {
        if *v > best_v {
            best_v = *v;
            best = i as i32;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_prompt_format() {
        let msgs = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        let prompt = build_chat_prompt(&msgs);
        assert!(prompt.contains("<|im_start|>user\nHello<|im_end|>"));
        assert!(prompt.contains("<|im_start|>assistant\n"));
    }

    #[test]
    fn test_model_decode_step() {
        if let Ok(model) = get_model() {
            let backend = model::get_backend();
            let ctx_params = build_context_params();
            let mut ctx = model.new_context(backend, ctx_params).expect("context creation");
            let tokens = model.str_to_token("Hello", AddBos::Always).expect("tokenize");
            let mut batch = LlamaBatch::new(16, 1);
            for (i, tok) in tokens.iter().enumerate() {
                let is_last = i + 1 == tokens.len();
                batch.add(*tok, i as i32, &[0], is_last).expect("batch add");
            }
            ctx.decode(&mut batch).expect("decode prompt");
            let last = (batch.n_tokens() - 1) as i32;
            let logits = ctx.get_logits_ith(last);
            assert!(!logits.is_empty());
        }
    }
}
