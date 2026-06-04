use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use crate::config_io::config_dir;

use super::{Tool, ToolContext};

pub struct CallMcpTool;

#[async_trait]
impl Tool for CallMcpTool {
    fn name(&self) -> &'static str {
        "call_mcp_tool"
    }

    fn description(&self) -> &'static str {
        "Invoke a tool on one of the configured Model Context Protocol (MCP) servers. \
         Use this to run specialized operations like searching actions or reading custom source files."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "server_name": {
                    "type": "string",
                    "description": "The name of the configured MCP server (e.g., 'actionbook')."
                },
                "tool_name": {
                    "type": "string",
                    "description": "The name of the tool to run on that server."
                },
                "arguments": {
                    "type": "object",
                    "description": "The parameters/arguments to pass to the tool."
                }
            },
            "required": ["server_name", "tool_name", "arguments"]
        })
    }

    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let server_name = args["server_name"].as_str().unwrap_or("").trim();
        let tool_name = args["tool_name"].as_str().unwrap_or("").trim();
        let tool_args = &args["arguments"];

        if server_name.is_empty() || tool_name.is_empty() {
            return Err("`server_name` and `tool_name` are required fields.".into());
        }

        // 1. Find command & args from mcp_registry.json
        let registry_path = config_dir().join("mcp_registry.json");
        if !registry_path.exists() {
            return Err("No MCP servers are configured. mcp_registry.json does not exist.".into());
        }

        let content = std::fs::read_to_string(&registry_path)
            .map_err(|e| format!("Failed to read mcp_registry.json: {}", e))?;
        let registry: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in mcp_registry.json: {}", e))?;

        // Support both "mcpServers" (frontend) and "mcp_servers" (backend fallback)
        let servers = registry.get("mcpServers")
            .or_else(|| registry.get("mcp_servers"))
            .and_then(|v| v.as_object())
            .ok_or_else(|| "No servers found in MCP registry. Use config to add one.".to_string())?;

        let server_config = servers.get(server_name)
            .ok_or_else(|| {
                let available = servers.keys().cloned().collect::<Vec<_>>().join(", ");
                format!("MCP server '{}' not found in registry. Available servers: {}", server_name, available)
            })?;

        let cmd = server_config.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("Missing 'command' in config for MCP server '{}'", server_name))?;

        let cmd_args: Vec<String> = server_config.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|val| val.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let mut env_map = HashMap::new();
        if let Some(env_obj) = server_config.get("env").and_then(|v| v.as_object()) {
            for (k, v) in env_obj {
                if let Some(s) = v.as_str() {
                    env_map.insert(k.clone(), s.to_string());
                }
            }
        }

        // Build the request JSON-RPC 2.0 object
        let call_req = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": tool_args
            },
            "id": 1
        });
        let call_str = serde_json::to_string(&call_req).unwrap();

        // 2. Spawn and execute
        match run_mcp(cmd, &cmd_args, &env_map, &call_str).await {
            Ok(resp_str) => {
                // Parse stdout and extract the content
                if let Ok(resp_json) = serde_json::from_str::<Value>(&resp_str) {
                    if let Some(err_val) = resp_json.get("error") {
                        return Err(format!("MCP tool error response: {}", err_val));
                    }
                    if let Some(result_val) = resp_json.get("result") {
                        // Extract text content from result
                        if let Some(content_arr) = result_val.get("content").and_then(|v| v.as_array()) {
                            let mut texts = Vec::new();
                            for block in content_arr {
                                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                        texts.push(text.to_string());
                                    }
                                }
                            }
                            if !texts.is_empty() {
                                return Ok(texts.join("\n"));
                            }
                        }
                        return Ok(serde_json::to_string_pretty(result_val).unwrap());
                    }
                }
                Ok(resp_str)
            }
            Err(e) => Err(format!("MCP communication error: {}", e))
        }
    }
}

async fn run_mcp(
    cmd: &str,
    args: &[String],
    env: &HashMap<String, String>,
    req_str: &str,
) -> Result<String, String> {
    let mut child = Command::new(cmd)
        .args(args)
        .envs(env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn MCP process '{}': {}", cmd, e))?;

    let mut stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    // 1. Initialize Request
    let init_req = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "llama-manager",
                "version": "1.0.0"
            }
        },
        "id": 0
    });
    let init_str = serde_json::to_string(&init_req).unwrap() + "\n";
    stdin.write_all(init_str.as_bytes()).await.map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())?;

    // 2. Read Initialize Response (id: 0)
    let mut init_responded = false;
    while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
        if line.trim().is_empty() { continue; }
        if let Ok(val) = serde_json::from_str::<Value>(&line) {
            if val.get("id").and_then(|v| v.as_i64()) == Some(0) {
                init_responded = true;
                break;
            }
        }
    }

    if !init_responded {
        let _ = child.kill().await;
        return Err("MCP server failed to respond to initialize request".into());
    }

    // 3. Send Initialized Notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    let init_notif_str = serde_json::to_string(&initialized_notification).unwrap() + "\n";
    stdin.write_all(init_notif_str.as_bytes()).await.map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())?;

    // 4. Send actual tools/call request (id: 1)
    let full_req_str = req_str.to_string() + "\n";
    stdin.write_all(full_req_str.as_bytes()).await.map_err(|e| e.to_string())?;
    stdin.flush().await.map_err(|e| e.to_string())?;

    // 5. Read response (id: 1)
    let mut response_str = String::new();
    while let Some(line) = reader.next_line().await.map_err(|e| e.to_string())? {
        if line.trim().is_empty() { continue; }
        if let Ok(val) = serde_json::from_str::<Value>(&line) {
            if val.get("id").and_then(|v| v.as_i64()) == Some(1) {
                response_str = line;
                break;
            }
        }
    }

    let _ = child.kill().await;

    if response_str.is_empty() {
        return Err("MCP server closed connection without returning a tool response".into());
    }

    Ok(response_str)
}
