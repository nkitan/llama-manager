//! Tool layer (L1). Every capability the agent can invoke implements the [`Tool`]
//! trait — one trait, uniform metadata + execution, so adding a tool is a single
//! file plus one line in [`ToolRegistry::builtin`]. MCP tools and future built-ins
//! share the same trait (see [[migration-conventions]]).
//!
//! `spawn_subagent` is advertised here (so the model knows it exists) but executed
//! specially by the engine, since spawning needs the agent runtime, not just I/O.

use std::path::PathBuf;

use async_trait::async_trait;
use serde_json::{Value, json};

mod files;
mod mcp_client;
mod shell;
mod web;

/// Ambient context handed to every tool. Intentionally small: tools do I/O, they
/// don't emit events or touch agent state.
pub struct ToolContext {
    pub searxng_url: Option<String>,
    pub config_dir: PathBuf,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    /// JSON Schema for the `parameters` object of the OpenAI tool spec.
    fn parameters(&self) -> Value;
    /// Side-effecting tools return true → the engine requires human approval (L4).
    fn is_sensitive(&self) -> bool {
        false
    }
    async fn run(&self, args: &Value, ctx: &ToolContext) -> Result<String, String>;
}

/// The name of the special, engine-handled sub-agent tool.
pub const SPAWN_SUBAGENT: &str = "spawn_subagent";

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// All built-in tools available to an agent.
    pub fn builtin() -> Self {
        Self {
            tools: vec![
                Box::new(web::WebSearch),
                Box::new(web::WebScrape),
                Box::new(files::GetTodos),
                Box::new(files::AddTodo),
                Box::new(files::CompleteTodo),
                Box::new(files::ReadNotes),
                Box::new(files::WriteNote),
                Box::new(files::GetCalendarEvents),
                Box::new(files::AddCalendarEvent),
                Box::new(files::DeleteCalendarEvent),
                Box::new(files::GetPlannerTasks),
                Box::new(files::AddPlannerTask),
                Box::new(files::UpdatePlannerTask),
                Box::new(files::DeletePlannerTask),
                Box::new(mcp_client::CallMcpTool),
                Box::new(shell::RunCommand),
            ],
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.name() == name).map(|b| &**b)
    }

    pub fn is_sensitive(&self, name: &str) -> bool {
        self.get(name).map(|t| t.is_sensitive()).unwrap_or(false)
    }

    /// Build the OpenAI-compatible `tools` array sent with each completion
    /// request, including the engine-handled `spawn_subagent` (L3).
    pub fn specs(&self) -> Vec<Value> {
        let mut specs: Vec<Value> = self
            .tools
            .iter()
            .map(|t| tool_spec(t.name(), t.description(), t.parameters()))
            .collect();
        specs.push(tool_spec(
            SPAWN_SUBAGENT,
            "Delegate a focused subtask to a specialized sub-agent that runs \
             autonomously and returns its result. Use for research or work that \
             benefits from an isolated context.",
            json!({
                "type": "object",
                "properties": {
                    "role": { "type": "string", "description": "Short role, e.g. 'researcher'." },
                    "prompt": { "type": "string", "description": "The subtask description." }
                },
                "required": ["prompt"]
            }),
        ));
        specs
    }
}

fn tool_spec(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": parameters,
        }
    })
}
