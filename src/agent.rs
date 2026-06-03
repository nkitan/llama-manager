use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use crate::todo::{TodoList, TodoEntry};
use crate::calendar::{CalendarState, CalendarEvent};
use crate::notes;

// ── Memory System ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MemoryStore {
    pub memories: Vec<String>,
}

// ── ChromaDB Helper Functions ────────────────────────────────────────────────
pub fn query_chroma_memories(query: &str) -> Vec<String> {
    let mut cmd = Command::new("uv");
    cmd.arg("run")
       .arg("/home/notroot/Work/llama-manager/chroma_memory.py")
       .arg("query")
       .arg("--text")
       .arg(query);
    
    let mut retrieved = Vec::new();
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout_str) {
                if let Some(arr) = val.as_array() {
                    for item in arr {
                        if let Some(doc) = item.get("document").and_then(|d| d.as_str()) {
                            retrieved.push(doc.to_string());
                        }
                    }
                }
            }
        }
    }
    retrieved
}

pub fn save_chroma_memory(text: &str, scope: &str) -> Result<(), String> {
    let mut cmd = Command::new("uv");
    cmd.arg("run")
       .arg("/home/notroot/Work/llama-manager/chroma_memory.py")
       .arg("add")
       .arg("--text")
       .arg(text)
       .arg("--scope")
       .arg(scope);
    
    let output = cmd.output().map_err(|e| format!("Failed to spawn command: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("ChromaDB failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

pub struct MemoryManager {
    pub global_path: PathBuf,
    pub project_path: PathBuf,
    pub global_store: MemoryStore,
    pub project_store: MemoryStore,
}

impl MemoryManager {
    pub fn new(base_dir: PathBuf) -> Self {
        let global_path = base_dir.join("global_memory.json");
        let project_path = base_dir.join(".llama-manager-memory.json");

        let global_store = Self::load_file(&global_path);
        let project_store = Self::load_file(&project_path);

        Self {
            global_path,
            project_path,
            global_store,
            project_store,
        }
    }

    fn load_file(path: &PathBuf) -> MemoryStore {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(store) = serde_json::from_str::<MemoryStore>(&content) {
                    return store;
                }
            }
        }
        MemoryStore::default()
    }

    pub fn save_global(&self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.global_store)
            .map_err(|e| format!("Failed to serialize global memory: {}", e))?;
        std::fs::write(&self.global_path, content)
            .map_err(|e| format!("Failed to write global memory: {}", e))?;
        Ok(())
    }

    pub fn save_project(&self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.project_store)
            .map_err(|e| format!("Failed to serialize project memory: {}", e))?;
        std::fs::write(&self.project_path, content)
            .map_err(|e| format!("Failed to write project memory: {}", e))?;
        Ok(())
    }

    pub fn add_global(&mut self, text: String) {
        if !text.trim().is_empty() && !self.global_store.memories.contains(&text) {
            self.global_store.memories.push(text.clone());
            let _ = self.save_global();
            let _ = save_chroma_memory(&text, "global");
        }
    }

    pub fn add_project(&mut self, text: String) {
        if !text.trim().is_empty() && !self.project_store.memories.contains(&text) {
            self.project_store.memories.push(text.clone());
            let _ = self.save_project();
            let _ = save_chroma_memory(&text, "project");
        }
    }

    pub fn clear_all(&mut self) {
        self.global_store.memories.clear();
        self.project_store.memories.clear();
        let _ = self.save_global();
        let _ = self.save_project();
    }
}

// ── MCP Tool Model ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum McpType {
    Local,
    Remote,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub tool_type: McpType,
    // Local tool execution params
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    // Remote tool HTTP details
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub ssl_verify_off: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct McpRegistry {
    pub tools: Vec<McpTool>,
}

impl McpRegistry {
    pub fn load(base_dir: PathBuf) -> Self {
        let path = base_dir.join("mcp_tools.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(registry) = serde_json::from_str::<McpRegistry>(&content) {
                    return registry;
                }
            }
        }
        // Provide some default tools
        let mut default_registry = McpRegistry::default();
        default_registry.tools.push(McpTool {
            name: "execute_local_cmd".to_string(),
            description: "Execute a command on the local system and return stdout".to_string(),
            tool_type: McpType::Local,
            command: Some("sh".to_string()),
            args: Some(vec!["-c".to_string(), "{{command}}".to_string()]),
            url: None,
            headers: None,
            ssl_verify_off: false,
        });
        default_registry
    }

    pub fn save(&self, base_dir: PathBuf) -> Result<(), String> {
        let path = base_dir.join("mcp_tools.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("IO Error: {}", e))?;
        Ok(())
    }

    pub async fn execute_tool(
        &self,
        name: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<String, String> {
        let tool = self.tools.iter().find(|t| t.name == name)
            .ok_or_else(|| format!("Tool '{}' not found in registry", name))?;

        match tool.tool_type {
            McpType::Local => {
                let cmd_str = tool.command.as_ref().ok_or_else(|| "Local tool has no command configured".to_string())?;
                let mut cmd = Command::new(cmd_str);

                if let Some(ref raw_args) = tool.args {
                    for arg in raw_args {
                        let mut formatted = arg.clone();
                        for (k, v) in arguments {
                            formatted = formatted.replace(&format!("{{{{{}}}}}", k), v);
                        }
                        cmd.arg(formatted);
                    }
                }

                let output = cmd.output().map_err(|e| format!("Failed to spawn local command: {}", e))?;
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

                if output.status.success() {
                    Ok(stdout)
                } else {
                    Err(format!("Command exited with status code: {:?}\nStdout: {}\nStderr: {}", output.status.code(), stdout, stderr))
                }
            }
            McpType::Remote => {
                let raw_url = tool.url.as_ref().ok_or_else(|| "Remote tool has no URL configured".to_string())?;
                let mut url = raw_url.clone();
                for (k, v) in arguments {
                    url = url.replace(&format!("{{{{{}}}}}", k), v);
                }

                let mut client_builder = reqwest::Client::builder();
                if tool.ssl_verify_off {
                    client_builder = client_builder.danger_accept_invalid_certs(true);
                }
                let client = client_builder.build().map_err(|e| format!("Failed to build HTTP client: {}", e))?;

                let mut request = client.post(&url);
                if let Some(ref headers) = tool.headers {
                    for (k, v) in headers {
                        request = request.header(k, v);
                    }
                }

                // Send arguments as JSON payload
                request = request.json(arguments);

                let response = request.send().await.map_err(|e| format!("HTTP request failed: {}", e))?;
                let text = response.text().await.map_err(|e| format!("Failed to read HTTP body: {}", e))?;
                Ok(text)
            }
        }
    }
}

// ── Web Scraping and Browser Use Layer ───────────────────────────────────────

pub struct WebBrowserLayer;

impl WebBrowserLayer {
    pub async fn search(query: &str, searxng_url: Option<&str>) -> Result<String, String> {
        // Try SearXNG if provided, otherwise fallback to DuckDuckGo html search
        if let Some(s_url) = searxng_url {
            let url = format!("{}/search?q={}&format=json", s_url, urlencoding::encode(query));
            let client = reqwest::Client::new();
            if let Ok(res) = client.get(&url).send().await {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    let mut summary = String::new();
                    if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
                        for (i, r) in results.iter().take(5).enumerate() {
                            let title = r.get("title").and_then(|t| t.as_str()).unwrap_or("");
                            let link = r.get("url").and_then(|u| u.as_str()).unwrap_or("");
                            let content = r.get("content").and_then(|c| c.as_str()).unwrap_or("");
                            summary.push_str(&format!("{}. [{}]({})\n   {}\n\n", i + 1, title, link, content));
                        }
                    }
                    if !summary.is_empty() {
                        return Ok(summary);
                    }
                }
            }
        }

        // DuckDuckGo fallback
        let ddg_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| e.to_string())?;

        match client.get(&ddg_url).send().await {
            Ok(res) => {
                let html = res.text().await.map_err(|e| e.to_string())?;
                // Parse simple links and snippets out of the HTML using basic string parsing
                let mut results = Vec::new();
                let mut cursor = 0;
                while let Some(start_idx) = html[cursor..].find("class=\"result__snippet\"") {
                    let idx = cursor + start_idx;
                    // Extract snippet
                    if let Some(snippet_start) = html[idx..].find('>') {
                        let sn_start = idx + snippet_start + 1;
                        if let Some(snippet_end) = html[sn_start..].find("</a>") {
                            let raw_snippet = &html[sn_start..sn_start + snippet_end];
                            // Strip HTML tags from snippet
                            let snippet = Self::strip_html_tags(raw_snippet);
                            
                            // Find corresponding title and link nearby (prior to snippet)
                            let slice_before = &html[cursor..idx];
                            let mut title = "Web Search Result".to_string();
                            let mut link = "".to_string();
                            if let Some(a_idx) = slice_before.rfind("class=\"result__url\"") {
                                let a_slice = &slice_before[a_idx..];
                                if let Some(href_start) = a_slice.find("href=\"") {
                                    let hr_start = href_start + 6;
                                    if let Some(href_end) = a_slice[hr_start..].find('"') {
                                        link = a_slice[hr_start..hr_start + href_end].to_string();
                                    }
                                }
                            }
                            if let Some(t_idx) = slice_before.rfind("class=\"result__a\"") {
                                let t_slice = &slice_before[t_idx..];
                                if let Some(title_start) = t_slice.find('>') {
                                    let title_st = title_start + 1;
                                    if let Some(title_end) = t_slice[title_st..].find("</a>") {
                                        title = Self::strip_html_tags(&t_slice[title_st..title_st + title_end]);
                                    }
                                }
                            }

                            results.push((title, link, snippet));
                        }
                    }
                    cursor = idx + 23; // Advance past current result__snippet tag
                    if results.len() >= 5 {
                        break;
                    }
                }

                if results.is_empty() {
                    Ok("Search completed, but no records returned. Try narrowing down your search terms.".to_string())
                } else {
                    let mut summary = String::new();
                    for (i, (title, link, snippet)) in results.iter().enumerate() {
                        summary.push_str(&format!("{}. [{}]({})\n   {}\n\n", i + 1, title, link, snippet));
                    }
                    Ok(summary)
                }
            }
            Err(e) => Err(format!("DuckDuckGo query failed: {}", e)),
        }
    }

    pub async fn scrape(url: &str) -> Result<String, String> {
        // Support browser-harness, camofox, cloakbrowser if present on command line
        // Otherwise fallback to HTTP reqwest with user agent
        let has_harness = Command::new("browser-harness").arg("--help").output().is_ok();
        let has_camofox = Command::new("camofox").arg("--help").output().is_ok();

        if has_harness {
            let output = Command::new("browser-harness")
                .arg("--url")
                .arg(url)
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
                }
            }
        } else if has_camofox {
            let output = Command::new("camofox")
                .arg("--url")
                .arg(url)
                .arg("--text-only")
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
                }
            }
        }

        // Fallback standard HTTP request
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| e.to_string())?;

        match client.get(url).send().await {
            Ok(res) => {
                let html = res.text().await.map_err(|e| e.to_string())?;
                let text = Self::strip_html_tags(&html);
                // Trim multiple spaces/newlines
                let clean_text: String = text
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<&str>>()
                    .join("\n");
                
                // Truncate to avoid blowing up context window
                if clean_text.len() > 10000 {
                    Ok(format!("{}... [Truncated for brevity]", &clean_text[..10000]))
                } else {
                    Ok(clean_text)
                }
            }
            Err(e) => Err(format!("Scraping failed: {}", e)),
        }
    }

    fn strip_html_tags(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_script_style = false;
        let mut current_tag = String::new();
        
        let chars: Vec<char> = html.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '<' {
                in_tag = true;
                current_tag.clear();
            } else if c == '>' {
                in_tag = false;
                let tag_lower = current_tag.to_lowercase();
                if tag_lower.starts_with("script") || tag_lower.starts_with("style") {
                    in_script_style = true;
                }
                if tag_lower.starts_with("/script") || tag_lower.starts_with("/style") {
                    in_script_style = false;
                }
            } else if in_tag {
                current_tag.push(c);
            } else if !in_script_style {
                result.push(c);
            }
            i += 1;
        }
        result
    }
}

// ── Agent Execution Engine ───────────────────────────────────────────────────

#[derive(serde::Deserialize, Clone)]
struct ChatMessage {
    content: Option<String>,
}

#[derive(serde::Deserialize, Clone)]
struct ChatChoice {
    message: Option<ChatMessage>,
}

#[derive(serde::Deserialize, Clone)]
struct ChatResponse {
    choices: Option<Vec<ChatChoice>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentResponse {
    pub text: String,
    pub logs: Vec<String>,
}

pub async fn run_agent_task(
    host: &str,
    port: u16,
    model: &str,
    task_prompt: &str,
    mcp_registry: &McpRegistry,
    memory_manager: &mut MemoryManager,
    searxng_url: Option<String>,
) -> Result<AgentResponse, String> {
    let mut logs = Vec::new();
    let mut messages = Vec::new();

    let monitor_path = crate::get_default_config_path().join("agent_activities.json");
    {
        let mut monitor_state = crate::planner::MonitorState::load(monitor_path.clone());
        monitor_state.add_event(model, &format!("Started task: {}", task_prompt), monitor_path.clone());
    }

    // Assemble memory block
    let mut memory_context = String::new();
    memory_context.push_str("Global Memory (Lessons & context learned from past sessions):\n");
    for m in &memory_manager.global_store.memories {
        memory_context.push_str(&format!("- {}\n", m));
    }
    memory_context.push_str("\nProject Memory (Lessons & context specific to this workspace):\n");
    for m in &memory_manager.project_store.memories {
        memory_context.push_str(&format!("- {}\n", m));
    }

    // Retrieve semantic memories from ChromaDB
    let chroma_mems = query_chroma_memories(task_prompt);
    let mut chroma_context = String::new();
    if !chroma_mems.is_empty() {
        chroma_context.push_str("\nRelevant memories retrieved from ChromaDB (Semantic Memory):\n");
        for m in chroma_mems {
            chroma_context.push_str(&format!("- {}\n", m));
        }
    }

    // Load TODO list items and inject into system prompt
    let project_todos = TodoList::load_project(memory_manager.project_path.parent().unwrap().to_path_buf());
    let global_todos = TodoList::load_global();
    
    let mut todos_context = String::new();
    todos_context.push_str("\nActive Todo Items:\n");
    todos_context.push_str("Global Todos:\n");
    for item in global_todos.items.iter().filter(|i| i.status == "pending") {
        todos_context.push_str(&format!("  - [{}] (ID: {}): {}\n", item.scope, item.id, item.text));
    }
    todos_context.push_str("Project Todos:\n");
    for item in project_todos.items.iter().filter(|i| i.status == "pending") {
        todos_context.push_str(&format!("  - [{}] (ID: {}): {}\n", item.scope, item.id, item.text));
    }

    // Load Quick Notes and inject into system prompt
    let project_notes = notes::load_notes("project", memory_manager.project_path.parent().unwrap().to_path_buf());
    let global_notes = notes::load_notes("global", memory_manager.project_path.parent().unwrap().to_path_buf());
    let mut notes_context = String::new();
    notes_context.push_str("\nQuick Notes:\n");
    notes_context.push_str(&format!("Global Notes: {}\n", if global_notes.is_empty() { "(empty)".to_string() } else { global_notes }));
    notes_context.push_str(&format!("Project Notes: {}\n", if project_notes.is_empty() { "(empty)".to_string() } else { project_notes }));

    // Assemble tool block
    let mut tools_context = String::new();
    tools_context.push_str("Available Tools:\n");
    tools_context.push_str("1. web_search(query): Query search engines for web pages.\n");
    tools_context.push_str("2. web_scrape(url): Extract textual data from a URL.\n");
    tools_context.push_str("3. spawn_subagent(prompt): Spawn a subagent to focus on a particular subtask asynchronously.\n");
    tools_context.push_str("4. add_todo(text, scope): Create a todo entry (scope can be 'project' or 'global').\n");
    tools_context.push_str("5. complete_todo(id, scope): Mark a todo entry as completed (scope can be 'project' or 'global').\n");
    tools_context.push_str("6. read_notes(scope): Read quick notes (scope can be 'project' or 'global').\n");
    tools_context.push_str("7. update_notes(content, scope): Update/write quick notes (scope can be 'project' or 'global').\n");
    tools_context.push_str("8. add_calendar_event(title, time, prompt): Schedule calendar event (time format YYYY-MM-DDTHH:MM:SS).\n");
    tools_context.push_str("9. list_calendar_events(): List scheduled calendar events.\n");
    for (i, t) in mcp_registry.tools.iter().enumerate() {
        tools_context.push_str(&format!("{}. {}: {}\n", i + 10, t.name, t.description));
    }

    let system_instructions = format!(
        "You are an Agent capable of autonomous task execution.\n\
         {memory_context}\n\
         {chroma_context}\n\
         {todos_context}\n\
         {notes_context}\n\
         {tools_context}\n\n\
         To execute tools, use the XML tags in your response:\n\
         <tool_call name=\"tool_name\">{{\"parameter\": \"value\"}}</tool_call>\n\n\
         If you need to spawn a subagent to focus on a particular subtask, call:\n\
         <tool_call name=\"spawn_subagent\">{{\"prompt\": \"subagent task description\"}}</tool_call>\n\n\
         When you have all information needed to solve the task or cannot execute any more tools, output your final reply directly without tool call tags."
    );

    messages.push(serde_json::json!({
        "role": "system",
        "content": system_instructions
    }));

    messages.push(serde_json::json!({
        "role": "user",
        "content": task_prompt
    }));

    let llm_url = format!("http://{}:{}/v1/chat/completions", host, port);
    let client = reqwest::Client::new();
    let mut step = 0;
    let max_steps = 6;
    let mut final_response = "No response generated".to_string();

    while step < max_steps {
        logs.push(format!("--- Step {} ---", step + 1));
        let payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.2,
        });

        let res = client.post(&llm_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Error contacting model server: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Model server returned error status: {}", res.status()));
        }

        let resp_json = res.json::<ChatResponse>().await
            .map_err(|e| format!("Failed to parse model response: {}", e))?;

        let assistant_msg = resp_json.choices
            .and_then(|c| c.first().cloned())
            .and_then(|c| c.message)
            .and_then(|m| m.content)
            .ok_or_else(|| "Empty response received from server".to_string())?;

        logs.push(format!("Agent Response: {}", assistant_msg));

        // Search for <tool_call ...> tag in assistant message
        if let Some(tool_call_start) = assistant_msg.find("<tool_call name=\"") {
            let inner_slice = &assistant_msg[tool_call_start + 17..];
            if let Some(name_end) = inner_slice.find('"') {
                let tool_name = &inner_slice[..name_end];
                if let Some(arg_start) = inner_slice[name_end..].find('>') {
                    let full_arg_start = name_end + arg_start + 1;
                    if let Some(tag_end) = inner_slice[full_arg_start..].find("</tool_call>") {
                        let args_str = &inner_slice[full_arg_start..full_arg_start + tag_end];
                        logs.push(format!("Executing Tool: {} with arguments: {}", tool_name, args_str.trim()));
                        {
                            let mut monitor_state = crate::planner::MonitorState::load(monitor_path.clone());
                            monitor_state.add_event(model, &format!("Executing Tool: {}", tool_name), monitor_path.clone());
                        }

                        // Try to parse arguments map
                        let mut arguments: HashMap<String, String> = HashMap::new();
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(args_str) {
                            if let Some(obj) = parsed.as_object() {
                                for (k, v) in obj {
                                    if let Some(s) = v.as_str() {
                                        arguments.insert(k.clone(), s.to_string());
                                    } else {
                                        arguments.insert(k.clone(), v.to_string());
                                    }
                                }
                            }
                        }

                        // Execute native or MCP tools
                        let execution_result = match tool_name {
                            "web_search" => {
                                let query = arguments.get("query").cloned().unwrap_or_default();
                                WebBrowserLayer::search(&query, searxng_url.as_deref()).await
                            }
                            "web_scrape" => {
                                let url = arguments.get("url").cloned().unwrap_or_default();
                                WebBrowserLayer::scrape(&url).await
                            }
                            "add_todo" => {
                                let text = arguments.get("text").cloned().unwrap_or_default();
                                let scope = arguments.get("scope").cloned().unwrap_or_else(|| "project".to_string());
                                let now = chrono::Local::now().to_rfc3339();
                                let id = format!("todo-{}", chrono::Local::now().timestamp_millis());
                                if scope == "global" {
                                    let mut list = TodoList::load_global();
                                    list.items.push(TodoEntry { id: id.clone(), text, scope, status: "pending".to_string(), created_at: now });
                                    let _ = list.save_global();
                                } else {
                                    let mut list = TodoList::load_project(memory_manager.project_path.parent().unwrap().to_path_buf());
                                    list.items.push(TodoEntry { id: id.clone(), text, scope, status: "pending".to_string(), created_at: now });
                                    let _ = list.save_project(memory_manager.project_path.parent().unwrap().to_path_buf());
                                }
                                Ok(format!("Added todo item with ID: {}", id))
                            }
                            "complete_todo" => {
                                let id = arguments.get("id").cloned().unwrap_or_default();
                                let scope = arguments.get("scope").cloned().unwrap_or_else(|| "project".to_string());
                                if scope == "global" {
                                    let mut list = TodoList::load_global();
                                    if let Some(item) = list.items.iter_mut().find(|i| i.id == id) {
                                        item.status = "completed".to_string();
                                        let _ = list.save_global();
                                        Ok(format!("Completed global todo: {}", id))
                                    } else {
                                        Err(format!("Global todo ID {} not found", id))
                                    }
                                } else {
                                    let mut list = TodoList::load_project(memory_manager.project_path.parent().unwrap().to_path_buf());
                                    if let Some(item) = list.items.iter_mut().find(|i| i.id == id) {
                                        item.status = "completed".to_string();
                                        let _ = list.save_project(memory_manager.project_path.parent().unwrap().to_path_buf());
                                        Ok(format!("Completed project todo: {}", id))
                                    } else {
                                        Err(format!("Project todo ID {} not found", id))
                                    }
                                }
                            }
                            "read_notes" => {
                                let scope = arguments.get("scope").cloned().unwrap_or_else(|| "project".to_string());
                                let n = notes::load_notes(&scope, memory_manager.project_path.parent().unwrap().to_path_buf());
                                Ok(format!("Notes (scope: {}):\n{}", scope, n))
                            }
                            "update_notes" => {
                                let content = arguments.get("content").cloned().unwrap_or_default();
                                let scope = arguments.get("scope").cloned().unwrap_or_else(|| "project".to_string());
                                match notes::save_notes(&scope, &content, memory_manager.project_path.parent().unwrap().to_path_buf()) {
                                    Ok(_) => Ok(format!("Successfully updated notes for scope: {}", scope)),
                                    Err(e) => Err(format!("Failed to save notes: {}", e))
                                }
                            }
                            "add_calendar_event" => {
                                let title = arguments.get("title").cloned().unwrap_or_default();
                                let time = arguments.get("time").cloned().unwrap_or_default();
                                let prompt = arguments.get("prompt").cloned().unwrap_or_default();
                                let mut state = CalendarState::load();
                                let id = format!("cal-{}", chrono::Local::now().timestamp_millis());
                                state.events.push(CalendarEvent {
                                    id: id.clone(),
                                    title,
                                    time,
                                    prompt,
                                    status: "pending".to_string(),
                                    result: None,
                                });
                                let _ = state.save();
                                Ok(format!("Added calendar event with ID: {}", id))
                            }
                            "list_calendar_events" => {
                                let state = CalendarState::load();
                                let mut res_str = String::from("Scheduled Calendar Events:\n");
                                for ev in &state.events {
                                    res_str.push_str(&format!("- [{}] (ID: {}): {} at {}, prompt: '{}'\n", 
                                        ev.status, ev.id, ev.title, ev.time, ev.prompt));
                                }
                                Ok(res_str)
                            }
                            "spawn_subagent" => {
                                let sub_prompt = arguments.get("prompt").cloned().unwrap_or_default();
                                logs.push(format!("[Subagent] Spawning subagent for subtask: {}", sub_prompt));
                                {
                                    let mut monitor_state = crate::planner::MonitorState::load(monitor_path.clone());
                                    monitor_state.add_event(model, &format!("Spawning subagent: {}", sub_prompt), monitor_path.clone());
                                }
                                // Subagents run recursively with same model and host, but step limit is lower
                                let mut box_memory = MemoryManager::new(memory_manager.global_path.parent().unwrap().to_path_buf());
                                match Box::pin(run_agent_task(
                                    host,
                                    port,
                                    model,
                                    &sub_prompt,
                                    mcp_registry,
                                    &mut box_memory,
                                    searxng_url.clone(),
                                )).await {
                                    Ok(sub_resp) => {
                                        for log in sub_resp.logs {
                                            logs.push(format!("[Subagent Log] {}", log));
                                        }
                                        Ok(format!("Subagent completed successfully. Output: {}", sub_resp.text))
                                    }
                                    Err(e) => Err(format!("Subagent failed: {}", e)),
                                }
                            }
                            _ => {
                                mcp_registry.execute_tool(tool_name, &arguments).await
                            }
                        };

                        let tool_result_str = match execution_result {
                            Ok(res) => {
                                logs.push(format!("Tool execution success: {}", if res.len() > 120 { format!("{}...", &res[..120]) } else { res.clone() }));
                                format!("Tool execution result:\n{}", res)
                            }
                            Err(e) => {
                                logs.push(format!("Tool execution error: {}", e));
                                format!("Tool execution error:\n{}", e)
                            }
                        };

                        // Add assistant message and tool response message
                        messages.push(serde_json::json!({
                            "role": "assistant",
                            "content": assistant_msg.clone()
                        }));
                        messages.push(serde_json::json!({
                            "role": "user",
                            "content": tool_result_str
                        }));

                        step += 1;
                        continue;
                    }
                }
            }
        }

        // No tool call or badly formatted tool call - treat as final response
        final_response = assistant_msg;
        break;
    }

    // --- Self-learning and Memory Reflection step ---
    logs.push("Performing self-learning reflection...".to_string());
    let reflection_prompt = format!(
        "Reflect on the task session and extract any critical lessons learned, project structures, \
         user habits, or optimization settings that should be saved for the next runs.\n\
         Task was: {}\n\
         Final response: {}\n\n\
         Respond in JSON format with two lists:\n\
         {{\n  \"global_lessons\": [\"item 1\", ...],\n  \"project_lessons\": [\"item 1\", ...]\n}}",
        task_prompt, final_response
    );

    let mut reflection_messages = messages.clone();
    reflection_messages.push(serde_json::json!({
        "role": "user",
        "content": reflection_prompt
    }));

    let reflection_payload = serde_json::json!({
        "model": model,
        "messages": reflection_messages,
        "temperature": 0.1,
    });

    if let Ok(res) = client.post(&llm_url).json(&reflection_payload).send().await {
        if res.status().is_success() {
            if let Ok(resp_json) = res.json::<ChatResponse>().await {
                if let Some(content) = resp_json.choices
                    .and_then(|c| c.first().cloned())
                    .and_then(|c| c.message)
                    .and_then(|m| m.content)
                {
                    // Clean content from JSON markers if LLM uses markdown blocks
                    let json_str = content.trim()
                        .trim_start_matches("```json")
                        .trim_start_matches("```")
                        .trim_end_matches("```")
                        .trim();
                    #[derive(serde::Deserialize)]
                    struct ReflectionLessons {
                        global_lessons: Option<Vec<String>>,
                        project_lessons: Option<Vec<String>>,
                    }
                    if let Ok(lessons) = serde_json::from_str::<ReflectionLessons>(json_str) {
                        if let Some(global) = lessons.global_lessons {
                            for g in global {
                                logs.push(format!("New Global Memory: {}", g));
                                memory_manager.add_global(g);
                            }
                        }
                        if let Some(project) = lessons.project_lessons {
                            for p in project {
                                logs.push(format!("New Project Memory: {}", p));
                                memory_manager.add_project(p);
                            }
                        }
                    }
                }
            }
        }
    }

    // Save interaction to ChromaDB so the agent learns/remembers
    let interaction_summary = format!("User task prompt: {}\nAgent response: {}", task_prompt, final_response);
    let _ = save_chroma_memory(&interaction_summary, "project");

    {
        let mut monitor_state = crate::planner::MonitorState::load(monitor_path.clone());
        monitor_state.add_event(model, "Task completed successfully", monitor_path.clone());
        monitor_state.set_idle(model, monitor_path.clone());
    }

    Ok(AgentResponse {
        text: final_response,
        logs,
    })
}
