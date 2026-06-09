use serde::Deserialize;
use std::fs;
use std::path::Path;
use shared::ipc::ScannedModel;

#[derive(Deserialize)]
struct SearxResult {
    title: Option<String>,
    url: Option<String>,
    content: Option<String>,
}

#[derive(Deserialize)]
struct SearxResponse {
    results: Option<Vec<SearxResult>>,
}

#[derive(serde::Serialize, Deserialize)]
struct LlmResponse {
    name: String,
    use_case: String,
    size: String,
    hf_link: String,
    github_link: String,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn clean_query_name(name: &str) -> String {
    let mut q = name.to_string();
    if q.ends_with(".gguf") {
        q.truncate(q.len() - 5);
    } else if q.ends_with(".safetensors") {
        q.truncate(q.len() - 12);
    } else if q.ends_with(".ckpt") {
        q.truncate(q.len() - 5);
    }

    // Remove common quantization / format suffixes
    let suffixes = [
        "-Q8_0", "-Q4_0", "-Q4_K_M", "-Q5_K_M", "-Q6_K", "-Q8_K_XL", "-Q4_K_XL", "-Q8_K_P",
        "-Q2_K_XL", "-Q2_K", "-BF16", "-f16", ".f16", ".dflash", "_Q8_0", "_Q4_0", "_Q4_K_M",
        "_Q5_K_M", "_Q6_K", "_Q8_K_XL", "_Q4_K_XL", ".Q8_0", ".Q4_0", ".Q4_K_M", ".Q5_K_M",
        ".Q6_K", ".Q8_K_XL", ".Q4_K_XL",
    ];
    for suffix in &suffixes {
        q = q.replace(suffix, "");
        q = q.replace(&suffix.to_lowercase(), "");
    }

    // Replace - and _ with space, but keep . when it separates digits
    q = q.replace("-", " ").replace("_", " ");
    let chars: Vec<char> = q.chars().collect();
    let mut cleaned = String::new();
    for i in 0..chars.len() {
        if chars[i] == '.' {
            let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
            let next_is_digit = i + 1 < chars.len() && chars[i + 1].is_ascii_digit();
            if prev_is_digit && next_is_digit {
                cleaned.push('.');
            } else {
                cleaned.push(' ');
            }
        } else {
            cleaned.push(chars[i]);
        }
    }
    cleaned.trim().to_string()
}

fn parse_json_from_llm(text: &str) -> Option<LlmResponse> {
    let trimmed = text.trim();
    if let Ok(res) = serde_json::from_str::<LlmResponse>(trimmed) {
        return Some(res);
    }

    // Try finding JSON block
    if let Some(start) = trimmed.find("```json") {
        let sub = &trimmed[start + 7..];
        if let Some(end) = sub.find("```") {
            let block = &sub[..end];
            if let Ok(res) = serde_json::from_str::<LlmResponse>(block.trim()) {
                return Some(res);
            }
        }
    }
    if let Some(start) = trimmed.find("```") {
        let sub = &trimmed[start + 3..];
        if let Some(end) = sub.find("```") {
            let block = &sub[..end];
            if let Ok(res) = serde_json::from_str::<LlmResponse>(block.trim()) {
                return Some(res);
            }
        }
    }

    // Fallback: look for opening { and closing }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            let block = &trimmed[start..=end];
            if let Ok(res) = serde_json::from_str::<LlmResponse>(block) {
                return Some(res);
            }
        }
    }

    None
}

// ── Public API ───────────────────────────────────────────────────────────────

pub fn load_index(path: &str) -> Vec<ScannedModel> {
    if let Ok(json) = fs::read_to_string(path) {
        if let Ok(index) = serde_json::from_str::<Vec<ScannedModel>>(&json) {
            return index;
        }
    }
    Vec::new()
}

pub fn save_index(path: &str, index: &[ScannedModel]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(index).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub struct ModelHierarchy {
    pub family: String,
    pub version: String,
    pub tags: Vec<String>,
}

pub fn parse_model_hierarchy(filename: &str) -> ModelHierarchy {
    let name_lower = filename.to_lowercase();

    // 1. Identify Family
    let family = if name_lower.contains("qwen") || name_lower.contains("qwopus") {
        "Qwen".to_string()
    } else if name_lower.contains("gemma") || name_lower.contains("supergemma") {
        "Gemma".to_string()
    } else if name_lower.contains("phi") {
        "Phi".to_string()
    } else if name_lower.contains("ernie") {
        "Ernie".to_string()
    } else if name_lower.contains("bge") {
        "BGE".to_string()
    } else if name_lower.contains("ternary-bonsai") || name_lower.contains("ternary_bonsai") {
        "Ternary Bonsai".to_string()
    } else if name_lower.contains("bonsai") {
        "Bonsai".to_string()
    } else if name_lower.contains("deepseek") {
        "DeepSeek".to_string()
    } else if name_lower.contains("lfm") {
        "LFM".to_string()
    } else if name_lower.contains("wan") {
        "Wan".to_string()
    } else if name_lower.contains("smoothmix") {
        "SmoothMix".to_string()
    } else if name_lower.contains("trellis") {
        "Trellis".to_string()
    } else if name_lower.contains("minicpm") {
        "MiniCPM".to_string()
    } else if name_lower.contains("granite") {
        "Granite".to_string()
    } else if name_lower.contains("vista") {
        "Vista".to_string()
    } else if name_lower.contains("kokoro") {
        "Kokoro".to_string()
    } else if name_lower.contains("glm") {
        "GLM".to_string()
    } else if name_lower.contains("ui-venus") || name_lower.contains("ui_venus") {
        "Ui Venus".to_string()
    } else if name_lower.contains("seed-coder") || name_lower.contains("seed_coder") {
        "Seed Coder".to_string()
    } else if name_lower.contains("zeta") {
        "Zeta".to_string()
    } else if name_lower.contains("sensenova") {
        "Sensenova".to_string()
    } else {
        let first_word = filename
            .split(|c| c == '-' || c == '_' || c == '.' || c == ' ')
            .next()
            .unwrap_or("Other");
        if first_word.len() > 2 && first_word.chars().all(|c| c.is_alphabetic()) {
            let mut c = first_word.chars();
            match c.next() {
                None => "Other".to_string(),
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
            }
        } else {
            "Other".to_string()
        }
    };

    // 2. Identify Version / Base Model Series
    let version = if family == "Qwen" {
        if name_lower.contains("qwen3-next") || name_lower.contains("coder-next") {
            "Qwen 3-Next".to_string()
        } else if name_lower.contains("qwen3.6")
            || name_lower.contains("qwen-3.6")
            || name_lower.contains("qwopus3.6")
        {
            "Qwen 3.6".to_string()
        } else if name_lower.contains("qwen3.5") || name_lower.contains("qwen-3.5") {
            "Qwen 3.5".to_string()
        } else if name_lower.contains("qwen3") {
            "Qwen 3.0".to_string()
        } else if name_lower.contains("qwen2.5") || name_lower.contains("qwen-2.5") {
            "Qwen 2.5".to_string()
        } else if name_lower.contains("qwen2") || name_lower.contains("qwen-2") {
            "Qwen 2.0".to_string()
        } else if name_lower.contains("qwen1.5") || name_lower.contains("qwen-1.5") {
            "Qwen 1.5".to_string()
        } else {
            "Qwen (Other)".to_string()
        }
    } else if family == "Gemma" {
        if name_lower.contains("gemma4")
            || name_lower.contains("gemma-4")
            || name_lower.contains("supergemma4")
        {
            "Gemma 4".to_string()
        } else if name_lower.contains("gemma3") || name_lower.contains("gemma-3") {
            "Gemma 3".to_string()
        } else if name_lower.contains("gemma2") || name_lower.contains("gemma-2") {
            "Gemma 2".to_string()
        } else {
            "Gemma (Other)".to_string()
        }
    } else if family == "Phi" {
        if name_lower.contains("phi-4") || name_lower.contains("phi4") {
            "Phi 4".to_string()
        } else if name_lower.contains("phi-3") || name_lower.contains("phi3") {
            "Phi 3".to_string()
        } else {
            "Phi (Other)".to_string()
        }
    } else if family == "Ernie" {
        if name_lower.contains("ernie-4") || name_lower.contains("ernie4") {
            "Ernie 4".to_string()
        } else if name_lower.contains("ernie-3") || name_lower.contains("ernie3") {
            "Ernie 3".to_string()
        } else {
            "Ernie (Other)".to_string()
        }
    } else if family == "Bonsai" || family == "Ternary Bonsai" {
        let prefix = if family == "Ternary Bonsai" {
            "Ternary Bonsai"
        } else {
            "Bonsai"
        };
        if name_lower.contains("1.7b") {
            format!("{} 1.7B", prefix)
        } else if name_lower.contains("4b") {
            format!("{} 4B", prefix)
        } else if name_lower.contains("8b") {
            format!("{} 8B", prefix)
        } else {
            prefix.to_string()
        }
    } else if family == "Ui Venus" {
        if name_lower.contains("1.5") {
            "Ui Venus 1.5".to_string()
        } else {
            "Ui Venus".to_string()
        }
    } else if family == "Seed Coder" {
        if name_lower.contains("8b") {
            "Seed Coder 8B".to_string()
        } else {
            "Seed Coder".to_string()
        }
    } else if family == "Zeta" {
        if name_lower.contains("2") {
            "Zeta 2".to_string()
        } else {
            "Zeta".to_string()
        }
    } else if family == "LFM" {
        if name_lower.contains("lfm2.5") || name_lower.contains("lfm-2.5") {
            "LFM 2.5".to_string()
        } else if name_lower.contains("lfm2") || name_lower.contains("lfm-2") {
            "LFM 2.0".to_string()
        } else {
            "LFM (Other)".to_string()
        }
    } else if family == "Granite" {
        if name_lower.contains("granite-4.1") || name_lower.contains("granite4.1") {
            "Granite 4.1".to_string()
        } else {
            "Granite 4.0".to_string()
        }
    } else if family == "Wan" {
        if name_lower.contains("wan2.2") || name_lower.contains("wan-2.2") {
            "Wan 2.2".to_string()
        } else {
            "Wan (Other)".to_string()
        }
    } else if family == "GLM" {
        if name_lower.contains("glm-4") || name_lower.contains("glm4") {
            "GLM 4".to_string()
        } else {
            "GLM".to_string()
        }
    } else {
        let clean = clean_query_name(filename);
        let words: Vec<&str> = clean.split_whitespace().collect();
        if words.len() >= 2 {
            let first_word = words[0];
            let capitalized = if first_word.eq_ignore_ascii_case(&family) {
                family.clone()
            } else {
                let mut c = first_word.chars();
                match c.next() {
                    None => first_word.to_string(),
                    Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                }
            };
            format!("{} {}", capitalized, words[1])
        } else if !words.is_empty() {
            let first_word = words[0];
            if first_word.eq_ignore_ascii_case(&family) {
                family.clone()
            } else {
                let mut c = first_word.chars();
                match c.next() {
                    None => first_word.to_string(),
                    Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                }
            }
        } else {
            family.clone()
        }
    };

    // 3. Extract Tags (Quantization, Uncensored, Thinking, etc.)
    let mut tags = Vec::new();

    if name_lower.contains(".gguf") {
        tags.push("GGUF".to_string());
    } else if name_lower.contains(".safetensors") {
        tags.push("Safetensors".to_string());
    } else if name_lower.contains(".ckpt") {
        tags.push("CKPT".to_string());
    }

    // Manual tokenizer for custom tags like quants or other features
    let separators = ['-', '.', ' ', '(', ')', '[', ']'];
    let parts: Vec<&str> = filename.split(&separators[..]).collect();

    for part in parts {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let p_lower = p.to_lowercase();

        if p_lower.contains("qwen")
            || p_lower.contains("gemma")
            || p_lower.contains("phi")
            || p_lower.contains("ernie")
            || p_lower.contains("bonsai")
        {
            continue;
        }
        if p_lower.ends_with('b')
            && p_lower.len() >= 2
            && p_lower.chars().nth(0).unwrap().is_ascii_digit()
        {
            continue;
        }

        let is_quant = if p_lower.starts_with('q') && p_lower.len() >= 2 {
            let second = p_lower.chars().nth(1).unwrap();
            second.is_ascii_digit()
        } else if p_lower.starts_with("iq") && p_lower.len() >= 3 {
            let third = p_lower.chars().nth(2).unwrap();
            third.is_ascii_digit()
        } else {
            p_lower == "bf16"
                || p_lower == "f16"
                || p_lower == "fp16"
                || p_lower == "f32"
                || p_lower == "fp32"
        };

        if is_quant {
            let tag_upper = p.to_uppercase();
            if !tags.contains(&tag_upper) {
                tags.push(tag_upper);
            }
        }
    }

    if name_lower.contains("uncensored") || name_lower.contains("unc") {
        tags.push("Uncensored".to_string());
    }
    if name_lower.contains("thinking") || name_lower.contains("reasoning") {
        tags.push("Thinking".to_string());
    }
    if name_lower.contains("abliterated") {
        tags.push("Abliterated".to_string());
    }
    if name_lower.contains("mtp") {
        tags.push("MTP".to_string());
    }
    if name_lower.contains("speculator") {
        tags.push("Speculator".to_string());
    }
    if name_lower.contains("embed") || name_lower.contains("embedding") {
        tags.push("Embedding".to_string());
    }
    if name_lower.contains("vision") || name_lower.contains("vl") {
        tags.push("Vision".to_string());
    }
    if name_lower.contains("ocr") {
        tags.push("OCR".to_string());
    }
    if name_lower.contains("distilled") || name_lower.contains("distil") {
        tags.push("Distilled".to_string());
    }
    if name_lower.contains("opus") {
        tags.push("Opus".to_string());
    }
    if name_lower.contains("coder") || name_lower.contains("code") {
        tags.push("Coder".to_string());
    }

    ModelHierarchy {
        family,
        version,
        tags,
    }
}

fn is_supported_model_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        matches!(
            extension.to_lowercase().as_str(),
            "gguf" | "safetensors" | "ckpt"
        )
    } else {
        false
    }
}

fn should_skip_entry(name: &str) -> bool {
    name.is_empty()
        || name.starts_with('.')
        || name == "dw.sh"
        || name == "generate_dw.py"
        || name == "link_models.sh"
        || name == "t"
}

fn scan_directory_recursive(path: &Path, models: &mut Vec<ScannedModel>) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if should_skip_entry(&name) {
                continue;
            }

            if path.is_dir() {
                scan_directory_recursive(&path, models);
                continue;
            }

            if !path.is_file() || !is_supported_model_file(&path) {
                continue;
            }

            let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let path_str = path.to_string_lossy().to_string();
            let clean = clean_query_name(&name);
            let size_info = format_size(size_bytes);
            let hierarchy = parse_model_hierarchy(&name);

            models.push(ScannedModel {
                path: path_str,
                filename: name,
                size_bytes,
                clean_name: clean,
                use_case: "Unknown".to_string(),
                hf_link: "".to_string(),
                github_link: "".to_string(),
                size_info,
                status: "pending_enrichment".to_string(),
                family: hierarchy.family,
                version: hierarchy.version,
                tags: hierarchy.tags,
            });
        }
    }
}

pub fn scan_directories(dirs: &[String]) -> Vec<ScannedModel> {
    let mut models = Vec::new();
    for dir_str in dirs {
        let dir_path = Path::new(dir_str);
        if !dir_path.is_dir() {
            continue;
        }
        scan_directory_recursive(dir_path, &mut models);
    }
    models
}

pub fn merge_indexes(scanned: Vec<ScannedModel>, existing: Vec<ScannedModel>) -> Vec<ScannedModel> {
    let mut merged = Vec::new();
    for mut s in scanned {
        if let Some(ext) = existing.iter().find(|e| e.path == s.path) {
            s.clean_name = ext.clean_name.clone();
            s.use_case = ext.use_case.clone();
            s.hf_link = ext.hf_link.clone();
            s.github_link = ext.github_link.clone();
            if ext.status == "enriched" || ext.status == "failed" || ext.status == "enriching" {
                s.status = ext.status.clone();
                s.size_info = ext.size_info.clone();
            }
            if !ext.family.is_empty() {
                s.family = ext.family.clone();
                s.version = ext.version.clone();
                s.tags = ext.tags.clone();
            }
        }
        merged.push(s);
    }
    merged
}

pub async fn get_target_model_with_host(
    host: &str,
    model_server_port: u16,
    active_model_path: &str,
) -> String {
    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/v1/models", host, model_server_port);
    if let Ok(res) = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
    {
        #[derive(Deserialize)]
        struct ModelStatus {
            value: Option<String>,
        }
        #[derive(Deserialize)]
        struct ModelInfo {
            id: Option<String>,
            status: Option<ModelStatus>,
        }
        #[derive(Deserialize)]
        struct ModelsList {
            data: Option<Vec<ModelInfo>>,
        }
        if let Ok(list) = res.json::<ModelsList>().await {
            if let Some(models) = list.data {
                // 1. Try to find a model that is already loaded
                for m in &models {
                    if let Some(ref st) = m.status {
                        if st.value.as_deref() == Some("loaded") {
                            if let Some(ref id) = m.id {
                                return id.clone();
                            }
                        }
                    }
                }
                // 2. Try to find a suitable text model
                let preferred = [
                    "bonsai-4b",
                    "bonsai-8b",
                    "bonsai-1.7b",
                    "nanbeige4.1-3b-Q8_0",
                    "qwen3.6-27b-Q4_K_M",
                ];
                for pref in &preferred {
                    if models.iter().any(|m| m.id.as_deref() == Some(*pref)) {
                        return pref.to_string();
                    }
                }
                // 3. Fallback to the first available model
                if let Some(first) = models.first() {
                    if let Some(ref id) = first.id {
                        return id.clone();
                    }
                }
            }
        }
    }

    // Default fallback
    if !active_model_path.is_empty() {
        if let Some(p) = Path::new(active_model_path).file_stem() {
            return p.to_string_lossy().to_string();
        }
    }

    "llama-server".to_string()
}

pub async fn get_target_model(model_server_port: u16, active_model_path: &str) -> String {
    get_target_model_with_host("127.0.0.1", model_server_port, active_model_path).await
}

fn map_pipeline_tag(tag: &str) -> String {
    match tag {
        "text-generation" => "Text Generation".to_string(),
        "text-to-image" => "Text-to-Image".to_string(),
        "text-to-video" => "Text-to-Video".to_string(),
        "image-to-text" => "Image-to-Text".to_string(),
        "image-text-to-text" => "Image-Text-to-Text".to_string(),
        "automatic-speech-recognition" => "ASR".to_string(),
        "text-to-speech" => "TTS".to_string(),
        "feature-extraction" => "Text Embedding".to_string(),
        "image-to-image" => "Image-to-Image".to_string(),
        "image-classification" => "Image Classification".to_string(),
        "translation" => "Translation".to_string(),
        "summarization" => "Summarization".to_string(),
        "question-answering" => "Question Answering".to_string(),
        "text-classification" => "Text Classification".to_string(),
        _ => {
            let parts: Vec<String> = tag
                .split('-')
                .map(|p| {
                    let mut chars = p.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(f) => {
                            f.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                        }
                    }
                })
                .collect();
            parts.join("-")
        }
    }
}

pub async fn enrich_model(
    model: &mut ScannedModel,
    searxng_url: &str,
    model_server_port: u16,
) -> Result<(), String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", reqwest::header::HeaderValue::from_static(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    ));
    headers.insert("Accept", reqwest::header::HeaderValue::from_static(
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
    ));
    headers.insert(
        "Accept-Language",
        reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let mut found_use_case = None;
    let mut found_hf_link = None;

    // 2. Query Hugging Face API
    let hf_query = clean_query_name(&model.filename);
    #[derive(Deserialize, Clone)]
    struct HfModelInfo {
        id: Option<String>,
        pipeline_tag: Option<String>,
    }

    if let Ok(res) = client
        .get("https://huggingface.co/api/models")
        .query(&[("search", &hf_query), ("limit", &"10".to_string())])
        .send()
        .await
    {
        if let Ok(models) = res.json::<Vec<HfModelInfo>>().await {
            for m in &models {
                if let Some(ref tag) = m.pipeline_tag {
                    found_use_case = Some(map_pipeline_tag(tag));
                    if let Some(ref hf_id) = m.id {
                        found_hf_link = Some(format!("https://huggingface.co/{}", hf_id));
                    }
                    break;
                }
            }
        }
    }

    if found_use_case.is_none() {
        let backup_query = format!("{} {}", model.family, model.version);
        if let Ok(res) = client
            .get("https://huggingface.co/api/models")
            .query(&[("search", &backup_query), ("limit", &"10".to_string())])
            .send()
            .await
        {
            if let Ok(models) = res.json::<Vec<HfModelInfo>>().await {
                for m in &models {
                    if let Some(ref tag) = m.pipeline_tag {
                        found_use_case = Some(map_pipeline_tag(tag));
                        if found_hf_link.is_none() {
                            if let Some(ref hf_id) = m.id {
                                found_hf_link = Some(format!("https://huggingface.co/{}", hf_id));
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    if let Some(uc) = found_use_case {
        model.use_case = uc;
    }
    if let Some(hl) = found_hf_link {
        model.hf_link = hl;
    }

    if model.use_case == "Unknown"
        || model.hf_link.is_empty()
        || model.clean_name == "Unknown"
        || model.clean_name.is_empty()
    {
        let clean = clean_query_name(&model.filename);
        let query = format!("{} model Hugging Face github", clean);

        let search_url = format!("{}/search", searxng_url.trim_end_matches('/'));
        if let Ok(res) = client
            .get(&search_url)
            .query(&[("q", &query), ("format", &"json".to_string())])
            .send()
            .await
        {
            if let Ok(response_text) = res.text().await {
                if let Ok(searx_res) = serde_json::from_str::<SearxResponse>(&response_text) {
                    let results = searx_res.results.unwrap_or_default();
                    if !results.is_empty() {
                        let mut search_results_text = String::new();
                        for (i, item) in results.iter().take(5).enumerate() {
                            let title = item.title.clone().unwrap_or_default();
                            let url = item.url.clone().unwrap_or_default();
                            let content = item.content.clone().unwrap_or_default();
                            search_results_text.push_str(&format!(
                                "Result {}:\nTitle: {}\nURL: {}\nSnippet: {}\n\n",
                                i + 1,
                                title,
                                url,
                                content
                            ));
                        }

                        let prompt = format!(
                            "You are a model metadata extractor. Your task is to analyze search results and extract accurate metadata about the AI model \"{}\".\n\n\
                            Search Results:\n\
                            {}\n\n\
                            Extract the following fields:\n\
                            1. Official Clean Name (e.g. \"BGE-M3\" or \"Qwen 3.5 Coder\")\n\
                            2. Primary Use-case (e.g. \"Text Embedding\", \"Text Generation\", \"ASR\", \"TTS\", \"Image Generation\", \"OCR\", \"Video Generation\", etc.)\n\
                            3. Parameter Size / File Info (e.g. \"8B\", \"300M\", \"24B\", or \"Unknown\")\n\
                            4. Hugging Face Link (the main Hugging Face repository URL, e.g. https://huggingface.co/...)\n\
                            5. GitHub Link (the source code repository URL, e.g. https://github.com/...)\n\n\
                            Respond ONLY with a JSON object in this format:\n\
                            {{\n\
                              \"name\": \"...\",\n\
                              \"use_case\": \"...\",\n\
                              \"size\": \"...\",\n\
                              \"hf_link\": \"...\",\n\
                              \"github_link\": \"...\"\n\
                            }}\n\
                            Do not include any other text, markdown formatting, or explanation. Just the raw JSON object.",
                            model.filename, search_results_text
                        );

                        let target_model =
                            get_target_model(model_server_port, &model.filename).await;
                        let llm_url =
                            format!("http://127.0.0.1:{}/v1/chat/completions", model_server_port);
                        let payload = serde_json::json!({
                            "model": target_model,
                            "messages": [
                                {
                                    "role": "user",
                                    "content": prompt
                                }
                            ],
                            "temperature": 0.1,
                            "response_format": { "type": "json_object" }
                        });

                        if let Ok(llm_res) = client.post(&llm_url).json(&payload).send().await {
                            if llm_res.status().is_success() {
                                #[derive(Deserialize, Clone)]
                                struct ChatMessage {
                                    content: Option<String>,
                                }
                                #[derive(Deserialize, Clone)]
                                struct ChatChoice {
                                    message: Option<ChatMessage>,
                                }
                                #[derive(Deserialize)]
                                struct ChatResponse {
                                    choices: Option<Vec<ChatChoice>>,
                                }

                                if let Ok(chat_response) = llm_res.json::<ChatResponse>().await {
                                    if let Some(content) = chat_response
                                        .choices
                                        .and_then(|c| c.first().cloned())
                                        .and_then(|c| c.message)
                                        .and_then(|m| m.content)
                                    {
                                        if let Some(parsed) = parse_json_from_llm(&content) {
                                            if !parsed.name.is_empty() && parsed.name != "..." {
                                                model.clean_name = parsed.name;
                                            }
                                            if model.use_case == "Unknown"
                                                && !parsed.use_case.is_empty()
                                                && parsed.use_case != "..."
                                            {
                                                model.use_case = parsed.use_case;
                                            }
                                            if model.hf_link.is_empty()
                                                && !parsed.hf_link.is_empty()
                                                && parsed.hf_link != "..."
                                            {
                                                model.hf_link = parsed.hf_link;
                                            }
                                            if !parsed.github_link.is_empty()
                                                && parsed.github_link != "..."
                                            {
                                                model.github_link = parsed.github_link;
                                            }
                                            if !parsed.size.is_empty()
                                                && parsed.size != "..."
                                                && parsed.size != "Unknown"
                                            {
                                                model.size_info = format!(
                                                    "{} ({})",
                                                    format_size(model.size_bytes),
                                                    parsed.size
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    model.status = "enriched".to_string();
    Ok(())
}
