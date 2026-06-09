//! The llama-server launch-configuration tabs. Every field reads/writes the
//! shared `ServerConfig` working copy via the `field_*!` macros (persisted on
//! commit). Enum selects are written inline.

use leptos::prelude::*;
use shared::config::{CacheType, LogFormat, PoolingType, RopeScaling, SplitMode};
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::{Card, SelectField};
use crate::state::AppCtx;
use crate::{field_bool, field_num, field_text};

#[component]
pub fn ModelTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    let pick_model = move |_| {
        spawn_local(async move {
            if let Ok(Some(p)) = api::pick_path(false).await {
                ctx.update_cfg(|c| c.model_path = p);
            }
        });
    };
    let pick_dir = move |_| {
        spawn_local(async move {
            if let Ok(Some(p)) = api::pick_path(true).await {
                ctx.update_cfg(|c| c.model_dir = p);
            }
        });
    };
    view! {
        <div style="display: flex; flex-direction: column; gap: 16px;">
            <Card title="Local Model">
                <div class="fields-grid">
                    {field_text!(ctx, exe_path, "llama-server Executable", "Path or command name")}
                    {field_text!(ctx, model_path, "Model Path (GGUF)", "Absolute path to a .gguf file")}
                    <div class="field">
                        <label class="field-label">"Model Operations"</label>
                        <div class="row-actions" style="margin-top: 0; height: 34px;">
                            <button class="btn secondary sm" on:click=pick_model>"Browse file…"</button>
                            <button class="btn secondary sm" on:click=pick_dir>"Browse dir…"</button>
                        </div>
                    </div>
                    {field_text!(ctx, model_dir, "Models Directory", "Used for HF downloads / scanning")}
                    {field_text!(ctx, model_alias, "Model Alias", "Name exposed via the API")}
                </div>
            </Card>
            <Card title="Remote / HuggingFace">
                <div class="fields-grid">
                    {field_text!(ctx, hf_repo, "HF Repo", "e.g. TheBloke/Llama-2-7B-GGUF")}
                    {field_text!(ctx, hf_file, "HF File", "Specific .gguf within the repo")}
                    {field_text!(ctx, hf_token, "HF Token", "For gated/private repos")}
                    {field_text!(ctx, model_url, "Model URL", "Direct download URL")}
                </div>
            </Card>
            <Card title="Prompting">
                <div class="fields-grid">
                    {field_text!(ctx, chat_template, "Chat Template", "Override the built-in template")}
                    {field_text!(ctx, system_prompt, "System Prompt", "Default system message")}
                </div>
            </Card>
        </div>
    }
}

#[component]
pub fn ContextTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    view! {
        <div style="display: flex; flex-direction: column; gap: 16px;">
            <Card title="Context & Batching">
                <div class="fields-grid">
                    {field_num!(ctx, ctx_size, u32, "Context Size", "-c · tokens of context")}
                    {field_num!(ctx, predict, i32, "Max Predict", "-n · -1 = unlimited")}
                    {field_num!(ctx, batch_size, u32, "Batch Size", "-b")}
                    {field_num!(ctx, ubatch_size, u32, "Micro-batch Size", "-ub")}
                    {field_num!(ctx, parallel, u32, "Parallel Sequences", "-np")}
                    {field_bool!(ctx, cont_batching, "Continuous Batching", "-cb")}
                </div>
            </Card>
        </div>
    }
}

#[component]
pub fn GpuTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    view! {
        <div style="display: flex; flex-direction: column; gap: 16px;">
            <Card title="Offloading">
                <div class="fields-grid">
                    {field_num!(ctx, gpu_layers, i32, "GPU Layers", "-ngl · -1 = all")}
                    <SelectField
                        label="Split Mode"
                        id="form-split_mode"
                        value=Signal::derive(move || ctx.config.get().split_mode.as_str().to_string())
                        options=vec![
                            ("none".into(), "None".into()),
                            ("layer".into(), "Layer".into()),
                            ("row".into(), "Row".into()),
                        ]
                        on_select=Callback::new(move |v: String| {
                            ctx.update_cfg(|c| c.split_mode = SplitMode::from_str(&v))
                        })
                    />
                    {field_text!(ctx, tensor_split, "Tensor Split", "-ts · e.g. 3,1")}
                    {field_num!(ctx, main_gpu, u32, "Main GPU", "-mg")}
                </div>
            </Card>
            <Card title="Memory">
                <div class="fields-grid">
                    {field_bool!(ctx, fit, "Auto-fit (--fit on)", "Auto-size offload to VRAM")}
                    {field_bool!(ctx, mlock, "mlock", "Lock model in RAM")}
                    {field_bool!(ctx, no_mmap, "No mmap", "Disable memory-mapping")}
                    {field_bool!(ctx, no_kv_offload, "No KV Offload", "-nkvo")}
                    <SelectField
                        label="KV Cache Type (K)"
                        id="form-cache_type_k"
                        value=Signal::derive(move || ctx.config.get().cache_type_k.as_str().to_string())
                        options=cache_opts()
                        on_select=Callback::new(move |v: String| {
                            ctx.update_cfg(|c| c.cache_type_k = CacheType::from_str(&v))
                        })
                    />
                    <SelectField
                        label="KV Cache Type (V)"
                        id="form-cache_type_v"
                        value=Signal::derive(move || ctx.config.get().cache_type_v.as_str().to_string())
                        options=cache_opts()
                        on_select=Callback::new(move |v: String| {
                            ctx.update_cfg(|c| c.cache_type_v = CacheType::from_str(&v))
                        })
                    />
                </div>
            </Card>
        </div>
    }
}

fn cache_opts() -> Vec<(String, String)> {
    vec![
        ("f16".into(), "F16".into()),
        ("q8_0".into(), "Q8_0".into()),
        ("q4_0".into(), "Q4_0".into()),
    ]
}

#[component]
pub fn PerformanceTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    view! {
        <div style="display: flex; flex-direction: column; gap: 16px;">
            <Card title="Threads">
                <div class="fields-grid">
                    {field_num!(ctx, threads, u32, "Threads", "-t · 0 = auto")}
                    {field_num!(ctx, threads_batch, u32, "Batch Threads", "-tb · 0 = auto")}
                </div>
            </Card>
            <Card title="Tuning">
                <div class="fields-grid">
                    {field_bool!(ctx, flash_attn, "Flash Attention", "-fa")}
                    {field_bool!(ctx, no_warmup, "No Warmup", "--no-warmup")}
                    {field_bool!(ctx, check_tensors, "Check Tensors", "--check-tensors")}
                </div>
            </Card>
        </div>
    }
}

#[component]
pub fn SamplingTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    view! {
        <div style="display: flex; flex-direction: column; gap: 16px;">
            <Card title="Core">
                <div class="fields-grid">
                    {field_num!(ctx, temp, f32, "Temperature", "--temp")}
                    {field_num!(ctx, top_k, u32, "Top-K", "--top-k")}
                    {field_num!(ctx, top_p, f32, "Top-P", "--top-p")}
                    {field_num!(ctx, min_p, f32, "Min-P", "--min-p")}
                    {field_num!(ctx, seed, i64, "Seed", "-1 = random")}
                </div>
            </Card>
            <Card title="Penalties">
                <div class="fields-grid">
                    {field_num!(ctx, repeat_penalty, f32, "Repeat Penalty", "")}
                    {field_num!(ctx, presence_penalty, f32, "Presence Penalty", "")}
                    {field_num!(ctx, frequency_penalty, f32, "Frequency Penalty", "")}
                </div>
            </Card>
            <Card title="Grammar">
                <div class="fields-grid">
                    {field_text!(ctx, grammar, "Grammar", "GBNF grammar string")}
                    {field_text!(ctx, grammar_file, "Grammar File", "Path to a .gbnf file")}
                </div>
            </Card>
        </div>
    }
}

#[component]
pub fn AdvancedTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    view! {
        <div style="display: flex; flex-direction: column; gap: 16px;">
            <Card title="RoPE">
                <div class="fields-grid">
                    <SelectField
                        label="RoPE Scaling"
                        id="form-rope_scaling"
                        value=Signal::derive(move || ctx.config.get().rope_scaling.as_str().to_string())
                        options=vec![
                            ("none".into(), "None".into()),
                            ("linear".into(), "Linear".into()),
                            ("yarn".into(), "YaRN".into()),
                        ]
                        on_select=Callback::new(move |v: String| {
                            ctx.update_cfg(|c| c.rope_scaling = RopeScaling::from_str(&v))
                        })
                    />
                    {field_num!(ctx, rope_freq_base, f32, "RoPE Freq Base", "")}
                    {field_num!(ctx, rope_freq_scale, f32, "RoPE Freq Scale", "")}
                    {field_num!(ctx, yarn_orig_ctx, u32, "YaRN Orig Context", "")}
                    {field_num!(ctx, yarn_ext_factor, f32, "YaRN Ext Factor", "")}
                    {field_num!(ctx, yarn_attn_factor, f32, "YaRN Attn Factor", "")}
                </div>
            </Card>
            <Card title="Speculative Decoding">
                <div class="fields-grid">
                    {field_text!(ctx, draft_model, "Draft Model", "Path to a small draft model")}
                    {field_num!(ctx, draft_gpu_layers, i32, "Draft GPU Layers", "-ngld")}
                    {field_num!(ctx, draft_tokens, u32, "Draft Tokens", "--draft")}
                </div>
            </Card>
        </div>
    }
}

#[component]
pub fn ApiTab() -> impl IntoView {
    let ctx = expect_context::<AppCtx>();
    view! {
        <div style="display: flex; flex-direction: column; gap: 16px;">
            <Card title="Security">
                <div class="fields-grid">
                    {field_text!(ctx, api_key, "API Key", "--api-key")}
                    {field_text!(ctx, api_key_file, "API Key File", "--api-key-file")}
                </div>
            </Card>
            <Card title="Endpoints">
                <div class="fields-grid">
                    {field_bool!(ctx, metrics, "Metrics", "--metrics")}
                    {field_bool!(ctx, slots, "Slots", "--slots")}
                    {field_text!(ctx, slot_save_path, "Slot Save Path", "--slot-save-path")}
                </div>
            </Card>
            <Card title="Embeddings">
                <div class="fields-grid">
                    {field_bool!(ctx, embedding, "Embedding Mode", "--embedding")}
                    <SelectField
                        label="Pooling"
                        id="form-pooling"
                        value=Signal::derive(move || ctx.config.get().pooling.as_str().to_string())
                        options=vec![
                            ("none".into(), "None".into()),
                            ("mean".into(), "Mean".into()),
                            ("cls".into(), "CLS".into()),
                            ("last".into(), "Last".into()),
                        ]
                        on_select=Callback::new(move |v: String| {
                            ctx.update_cfg(|c| c.pooling = PoolingType::from_str(&v))
                        })
                    />
                </div>
            </Card>
            <Card title="Logging">
                <div class="fields-grid">
                    <SelectField
                        label="Log Format"
                        id="form-log_format"
                        value=Signal::derive(move || ctx.config.get().log_format.as_str().to_string())
                        options=vec![("text".into(), "Text".into()), ("json".into(), "JSON".into())]
                        on_select=Callback::new(move |v: String| {
                            ctx.update_cfg(|c| c.log_format = LogFormat::from_str(&v))
                        })
                    />
                    {field_bool!(ctx, verbose, "Verbose", "-v")}
                </div>
            </Card>
        </div>
    }
}
