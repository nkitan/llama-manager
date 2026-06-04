//! `ServerConfig` — the single application configuration object.
//!
//! Ported from the legacy app's `config.rs`. The filesystem load/save helpers
//! were intentionally *not* moved here (they need `std::fs`, which is unavailable
//! on wasm); they live in `src-tauri/src/config_io.rs`. Everything here is pure
//! and wasm-safe: the type definitions, defaults, and the `to_args` /
//! `command_preview` builders the UI and backend both rely on.

use serde::{Deserialize, Serialize};

fn default_searxng_url() -> String {
    "http://localhost:8888".to_string()
}
fn default_ui_transparency() -> f32 {
    0.1
}
fn default_ui_background_color() -> String {
    "#0f172a".to_string()
}
fn default_ui_blur() -> bool {
    true
}
fn default_ui_blur_intensity() -> u32 {
    30
}
fn default_log_level() -> String {
    "INFO".to_string()
}
fn default_theme() -> String {
    "default".to_string()
}
fn default_ui_text_color() -> String {
    "#f8fafc".to_string()
}
fn default_ui_accent_color() -> String {
    "#6366f1".to_string()
}
fn default_ui_card_bg() -> String {
    "rgba(255, 255, 255, 0.02)".to_string()
}
fn default_ui_sidebar_bg() -> String {
    "linear-gradient(135deg, rgba(255, 255, 255, 0.04) 0%, rgba(255, 255, 255, 0.01) 100%)"
        .to_string()
}
fn default_ui_border_color() -> String {
    "rgba(255, 255, 255, 0.08)".to_string()
}
fn default_ui_font_family() -> String {
    "Inter".to_string()
}
fn default_dark_mode() -> bool {
    true
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomTheme {
    pub name: String,
    pub background_color: String,
    pub text_color: String,
    pub accent_color: String,
    pub card_bg: String,
    pub sidebar_bg: String,
    pub border_color: String,
    pub font_family: String,
    pub transparency: f32,
    pub blur: bool,
    pub blur_intensity: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelOverride {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub model: String,
}

impl Default for ModelOverride {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".into(),
            port: 8080,
            model: String::new(),
        }
    }
}


// ── Enums ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SplitMode {
    None,
    Layer,
    Row,
}
impl SplitMode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Layer => "layer",
            Self::Row => "row",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "none" => Self::None,
            "row" => Self::Row,
            _ => Self::Layer,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CacheType {
    F16,
    Q8_0,
    Q4_0,
}
impl CacheType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::F16 => "f16",
            Self::Q8_0 => "q8_0",
            Self::Q4_0 => "q4_0",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "q8_0" => Self::Q8_0,
            "q4_0" => Self::Q4_0,
            _ => Self::F16,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RopeScaling {
    None,
    Linear,
    Yarn,
}
impl RopeScaling {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Linear => "linear",
            Self::Yarn => "yarn",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "linear" => Self::Linear,
            "yarn" => Self::Yarn,
            _ => Self::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
}
impl LogFormat {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "json" => Self::Json,
            _ => Self::Text,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PoolingType {
    None,
    Mean,
    Cls,
    Last,
}
impl PoolingType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Mean => "mean",
            Self::Cls => "cls",
            Self::Last => "last",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "mean" => Self::Mean,
            "cls" => Self::Cls,
            "last" => Self::Last,
            _ => Self::None,
        }
    }
}

// ── LoRA Adapter ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoraAdapter {
    pub path: String,
    pub scale: f32,
}

// ── Server Configuration ────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    // ─ Executable
    pub exe_path: String,

    // ─ Model
    pub model_path: String,
    pub model_dir: String,
    pub model_alias: String,
    pub model_url: String,
    pub hf_repo: String,
    pub hf_file: String,
    pub hf_token: String,
    pub chat_template: String,
    pub system_prompt: String,

    // ─ Server
    pub host: String,
    pub port: u16,
    pub timeout: u32,
    pub threads_http: u32,

    // ─ Context & Batching
    pub ctx_size: u32,
    pub predict: i32,
    pub batch_size: u32,
    pub ubatch_size: u32,
    pub parallel: u32,
    pub cont_batching: bool,

    // ─ GPU & Memory
    pub gpu_layers: i32,
    pub split_mode: SplitMode,
    pub tensor_split: String,
    pub main_gpu: u32,
    pub fit: bool,
    pub mlock: bool,
    pub no_mmap: bool,
    pub no_kv_offload: bool,
    pub cache_type_k: CacheType,
    pub cache_type_v: CacheType,

    // ─ Performance
    pub threads: u32,
    pub threads_batch: u32,
    pub flash_attn: bool,
    pub no_warmup: bool,
    pub check_tensors: bool,

    // ─ Sampling
    pub temp: f32,
    pub top_k: u32,
    pub top_p: f32,
    pub min_p: f32,
    pub repeat_penalty: f32,
    pub presence_penalty: f32,
    pub frequency_penalty: f32,
    pub seed: i64,
    pub grammar: String,
    pub grammar_file: String,

    // ─ RoPE
    pub rope_scaling: RopeScaling,
    pub rope_freq_base: f32,
    pub rope_freq_scale: f32,
    pub yarn_orig_ctx: u32,
    pub yarn_ext_factor: f32,
    pub yarn_attn_factor: f32,
    pub yarn_beta_fast: f32,
    pub yarn_beta_slow: f32,

    // ─ LoRA
    pub lora_adapters: Vec<LoraAdapter>,

    // ─ Speculative Decoding
    pub draft_model: String,
    pub draft_gpu_layers: i32,
    pub draft_tokens: u32,

    // ─ API & Security
    pub api_key: String,
    pub api_key_file: String,
    pub metrics: bool,
    pub slots: bool,
    pub slot_save_path: String,

    // ─ Embedding
    pub embedding: bool,
    pub pooling: PoolingType,

    // ─ Logging
    pub log_format: LogFormat,
    pub verbose: bool,

    // ─ Model Indexing / Scan Settings
    #[serde(default)]
    pub model_scan_dirs: Vec<String>,
    #[serde(default = "default_searxng_url")]
    pub searxng_url: String,

    // ─ UI Settings
    #[serde(default = "default_ui_transparency")]
    pub ui_transparency: f32,
    #[serde(default = "default_ui_background_color")]
    pub ui_background_color: String,
    #[serde(default = "default_ui_blur")]
    pub ui_blur: bool,
    #[serde(default = "default_ui_blur_intensity")]
    pub ui_blur_intensity: u32,
    #[serde(default = "default_log_level")]
    pub app_log_level: String,
    #[serde(default = "default_theme")]
    pub theme_name: String,
    #[serde(default = "default_ui_text_color")]
    pub ui_text_color: String,
    #[serde(default = "default_ui_accent_color")]
    pub ui_accent_color: String,
    #[serde(default = "default_ui_card_bg")]
    pub ui_card_bg: String,
    #[serde(default = "default_ui_sidebar_bg")]
    pub ui_sidebar_bg: String,
    #[serde(default = "default_ui_border_color")]
    pub ui_border_color: String,
    #[serde(default = "default_ui_font_family")]
    pub ui_font_family: String,
    #[serde(default)]
    pub sidebar_favorites: Vec<String>,
    #[serde(default)]
    pub custom_themes: Vec<CustomTheme>,
    #[serde(default)]
    pub override_planner: ModelOverride,
    #[serde(default)]
    pub override_calendar: ModelOverride,
    #[serde(default)]
    pub override_memory: ModelOverride,
    #[serde(default)]
    pub override_research: ModelOverride,
    #[serde(default)]
    pub override_compare: ModelOverride,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            exe_path: "llama-server".into(),

            // Model
            model_path: String::new(),
            model_dir: String::new(),
            model_alias: String::new(),
            model_url: String::new(),
            hf_repo: String::new(),
            hf_file: String::new(),
            hf_token: String::new(),
            chat_template: String::new(),
            system_prompt: String::new(),

            // Server
            host: "127.0.0.1".into(),
            port: 8080,
            timeout: 0,
            threads_http: 0,

            // Context & Batching
            ctx_size: 8192,
            predict: -1,
            batch_size: 2048,
            ubatch_size: 512,
            parallel: 1,
            cont_batching: true,

            // GPU & Memory  (sane defaults)
            gpu_layers: -1, // offload all layers
            split_mode: SplitMode::Layer,
            tensor_split: String::new(),
            main_gpu: 0,
            fit: true, // --fit on  ← sane default
            mlock: false,
            no_mmap: false,
            no_kv_offload: false,
            cache_type_k: CacheType::F16,
            cache_type_v: CacheType::F16,

            // Performance  (sane defaults)
            threads: 0,       // 0 = auto-detect
            threads_batch: 0, // 0 = auto-detect
            flash_attn: true, // -fa  ← sane default
            no_warmup: false,
            check_tensors: false,

            // Sampling
            temp: 0.8,
            top_k: 40,
            top_p: 0.95,
            min_p: 0.05,
            repeat_penalty: 1.0,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            seed: -1,
            grammar: String::new(),
            grammar_file: String::new(),

            // RoPE
            rope_scaling: RopeScaling::None,
            rope_freq_base: 0.0,
            rope_freq_scale: 0.0,
            yarn_orig_ctx: 0,
            yarn_ext_factor: -1.0,
            yarn_attn_factor: 1.0,
            yarn_beta_fast: 32.0,
            yarn_beta_slow: 1.0,

            // LoRA
            lora_adapters: Vec::new(),

            // Speculative Decoding
            draft_model: String::new(),
            draft_gpu_layers: -1,
            draft_tokens: 5,

            // API & Security
            api_key: String::new(),
            api_key_file: String::new(),
            metrics: false,
            slots: false,
            slot_save_path: String::new(),

            // Embedding
            embedding: false,
            pooling: PoolingType::None,

            // Logging
            log_format: LogFormat::Text,
            verbose: false,

            // Model Indexing / Scan Settings
            model_scan_dirs: Vec::new(),
            searxng_url: "http://localhost:8888".into(),

            // UI Settings
            ui_transparency: 0.1,
            ui_background_color: "#0f172a".into(),
            ui_blur: true,
            ui_blur_intensity: 30,
            app_log_level: "INFO".into(),
            theme_name: "default".into(),
            ui_text_color: "#f8fafc".into(),
            ui_accent_color: "#6366f1".into(),
            ui_card_bg: "rgba(255, 255, 255, 0.06)".into(),
            ui_sidebar_bg: "linear-gradient(135deg, rgba(255, 255, 255, 0.08) 0%, rgba(255, 255, 255, 0.02) 100%)".into(),
            ui_border_color: "rgba(255, 255, 255, 0.12)".into(),
            ui_font_family: "Inter".into(),
            sidebar_favorites: Vec::new(),
            custom_themes: Vec::new(),
            override_planner: ModelOverride::default(),
            override_calendar: ModelOverride::default(),
            override_memory: ModelOverride::default(),
            override_research: ModelOverride::default(),
            override_compare: ModelOverride::default(),
            dark_mode: true,
        }
    }
}

impl ServerConfig {
    /// Build the CLI argument list for `llama-server`.
    pub fn to_args(&self) -> Vec<String> {
        let mut a = Vec::new();

        // ── Model ───────────────────────────────────────────────────
        if !self.model_path.is_empty() {
            a.extend(["-m".into(), self.model_path.clone()]);
        }
        if !self.model_dir.is_empty() {
            a.extend(["--models-dir".into(), self.model_dir.clone()]);
        }
        if !self.model_alias.is_empty() {
            a.extend(["-a".into(), self.model_alias.clone()]);
        }
        if !self.model_url.is_empty() {
            a.extend(["-mu".into(), self.model_url.clone()]);
        }
        if !self.hf_repo.is_empty() {
            a.extend(["-hfr".into(), self.hf_repo.clone()]);
        }
        if !self.hf_file.is_empty() {
            a.extend(["-hff".into(), self.hf_file.clone()]);
        }
        if !self.hf_token.is_empty() {
            a.extend(["-hft".into(), self.hf_token.clone()]);
        }
        if !self.chat_template.is_empty() {
            a.extend(["--chat-template".into(), self.chat_template.clone()]);
        }
        if !self.system_prompt.is_empty() {
            a.extend(["-sp".into(), self.system_prompt.clone()]);
        }

        // ── Server ──────────────────────────────────────────────────
        a.extend(["--host".into(), self.host.clone()]);
        a.extend(["--port".into(), self.port.to_string()]);
        if self.timeout > 0 {
            a.extend(["--timeout".into(), self.timeout.to_string()]);
        }
        if self.threads_http > 0 {
            a.extend(["--threads-http".into(), self.threads_http.to_string()]);
        }

        // ── Context ─────────────────────────────────────────────────
        a.extend(["-c".into(), self.ctx_size.to_string()]);
        if self.predict != -1 {
            a.extend(["-n".into(), self.predict.to_string()]);
        }
        a.extend(["-b".into(), self.batch_size.to_string()]);
        a.extend(["-ub".into(), self.ubatch_size.to_string()]);
        if self.parallel > 1 {
            a.extend(["-np".into(), self.parallel.to_string()]);
        }
        if self.cont_batching {
            a.push("-cb".into());
        }

        // ── GPU & Memory ────────────────────────────────────────────
        if self.gpu_layers != 0 {
            a.extend(["-ngl".into(), self.gpu_layers.to_string()]);
        }
        if self.split_mode != SplitMode::Layer {
            a.extend(["-sm".into(), self.split_mode.as_str().into()]);
        }
        if !self.tensor_split.is_empty() {
            a.extend(["-ts".into(), self.tensor_split.clone()]);
        }
        if self.main_gpu > 0 {
            a.extend(["-mg".into(), self.main_gpu.to_string()]);
        }
        if self.fit {
            a.extend(["--fit".into(), "on".into()]);
        }
        if self.mlock {
            a.push("--mlock".into());
        }
        if self.no_mmap {
            a.push("--no-mmap".into());
        }
        if self.no_kv_offload {
            a.push("-nkvo".into());
        }
        if self.cache_type_k != CacheType::F16 {
            a.extend(["-ctk".into(), self.cache_type_k.as_str().into()]);
        }
        if self.cache_type_v != CacheType::F16 {
            a.extend(["-ctv".into(), self.cache_type_v.as_str().into()]);
        }

        // ── Performance ─────────────────────────────────────────────
        if self.threads > 0 {
            a.extend(["-t".into(), self.threads.to_string()]);
        }
        if self.threads_batch > 0 {
            a.extend(["-tb".into(), self.threads_batch.to_string()]);
        }
        // -fa requires a value: on, off, or auto
        a.extend([
            "-fa".into(),
            if self.flash_attn { "on" } else { "off" }.into(),
        ]);
        if self.no_warmup {
            a.push("--no-warmup".into());
        }
        if self.check_tensors {
            a.push("--check-tensors".into());
        }

        // ── Sampling ────────────────────────────────────────────────
        a.extend(["--temp".into(), format!("{:.2}", self.temp)]);
        a.extend(["--top-k".into(), self.top_k.to_string()]);
        a.extend(["--top-p".into(), format!("{:.2}", self.top_p)]);
        a.extend(["--min-p".into(), format!("{:.2}", self.min_p)]);
        if (self.repeat_penalty - 1.0).abs() > 0.001 {
            a.extend([
                "--repeat-penalty".into(),
                format!("{:.2}", self.repeat_penalty),
            ]);
        }
        if self.presence_penalty.abs() > 0.001 {
            a.extend([
                "--presence-penalty".into(),
                format!("{:.2}", self.presence_penalty),
            ]);
        }
        if self.frequency_penalty.abs() > 0.001 {
            a.extend([
                "--frequency-penalty".into(),
                format!("{:.2}", self.frequency_penalty),
            ]);
        }
        if self.seed != -1 {
            a.extend(["-s".into(), self.seed.to_string()]);
        }
        if !self.grammar.is_empty() {
            a.extend(["--grammar".into(), self.grammar.clone()]);
        }
        if !self.grammar_file.is_empty() {
            a.extend(["--grammar-file".into(), self.grammar_file.clone()]);
        }

        // ── RoPE ────────────────────────────────────────────────────
        if self.rope_scaling != RopeScaling::None {
            a.extend(["--rope-scaling".into(), self.rope_scaling.as_str().into()]);
        }
        if self.rope_freq_base > 0.0 {
            a.extend(["--rope-freq-base".into(), self.rope_freq_base.to_string()]);
        }
        if self.rope_freq_scale > 0.0 {
            a.extend(["--rope-freq-scale".into(), self.rope_freq_scale.to_string()]);
        }
        if self.rope_scaling == RopeScaling::Yarn {
            if self.yarn_orig_ctx > 0 {
                a.extend(["--yarn-orig-ctx".into(), self.yarn_orig_ctx.to_string()]);
            }
            if (self.yarn_ext_factor + 1.0).abs() > 0.001 {
                a.extend(["--yarn-ext-factor".into(), self.yarn_ext_factor.to_string()]);
            }
            if (self.yarn_attn_factor - 1.0).abs() > 0.001 {
                a.extend([
                    "--yarn-attn-factor".into(),
                    self.yarn_attn_factor.to_string(),
                ]);
            }
        }

        // ── LoRA ────────────────────────────────────────────────────
        for lora in &self.lora_adapters {
            if !lora.path.is_empty() {
                if (lora.scale - 1.0).abs() > 0.001 {
                    a.extend([
                        "--lora-scaled".into(),
                        lora.path.clone(),
                        format!("{:.2}", lora.scale),
                    ]);
                } else {
                    a.extend(["--lora".into(), lora.path.clone()]);
                }
            }
        }

        // ── Speculative Decoding ────────────────────────────────────
        if !self.draft_model.is_empty() {
            a.extend(["-md".into(), self.draft_model.clone()]);
            if self.draft_gpu_layers != 0 {
                a.extend(["-ngld".into(), self.draft_gpu_layers.to_string()]);
            }
            a.extend(["--draft".into(), self.draft_tokens.to_string()]);
        }

        // ── API & Security ──────────────────────────────────────────
        if !self.api_key.is_empty() {
            a.extend(["--api-key".into(), self.api_key.clone()]);
        }
        if !self.api_key_file.is_empty() {
            a.extend(["--api-key-file".into(), self.api_key_file.clone()]);
        }
        if self.metrics {
            a.push("--metrics".into());
        }
        if self.slots {
            a.push("--slots".into());
        }
        if !self.slot_save_path.is_empty() {
            a.extend(["--slot-save-path".into(), self.slot_save_path.clone()]);
        }

        // ── Embedding ───────────────────────────────────────────────
        if self.embedding {
            a.push("--embedding".into());
            if self.pooling != PoolingType::None {
                a.extend(["--pooling".into(), self.pooling.as_str().into()]);
            }
        }

        // ── Logging ─────────────────────────────────────────────────
        if self.log_format != LogFormat::Text {
            a.extend(["--log-format".into(), self.log_format.as_str().into()]);
        }
        if self.verbose {
            a.push("-v".into());
        }

        a
    }

    /// Human-readable command string for display.
    pub fn command_preview(&self) -> String {
        let args = self.to_args();
        let mut parts = vec![self.exe_path.clone()];
        parts.extend(args);
        parts.join(" ")
    }
}
