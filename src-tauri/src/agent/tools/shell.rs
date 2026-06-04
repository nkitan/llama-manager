//! `run_command` — execute a shell command. This is the canonical *sensitive*
//! tool: it always requires human approval (L4) and runs sandboxed by a timeout
//! and an output cap. The working directory is pinned to the config dir; the
//! command never inherits an interactive shell.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::process::Command;

use super::{Tool, ToolContext};

/// Hard ceiling so a runaway command can't hang the agent.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_OUTPUT: usize = 8_000;

pub struct RunCommand;

#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &'static str {
        "run_command"
    }
    fn description(&self) -> &'static str {
        "Run a shell command in the project directory and return its stdout/stderr. \
         Requires user approval."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, ctx: &ToolContext) -> Result<String, String> {
        let command = args["command"].as_str().unwrap_or("").trim();
        if command.is_empty() {
            return Err("`command` is required".into());
        }

        // `sh -c` keeps it a single, non-interactive invocation pinned to the
        // project dir — no inherited TTY, bounded by COMMAND_TIMEOUT.
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command)
            .current_dir(&ctx.config_dir)
            .kill_on_drop(true);

        let output = tokio::time::timeout(COMMAND_TIMEOUT, cmd.output())
            .await
            .map_err(|_| format!("command timed out after {}s", COMMAND_TIMEOUT.as_secs()))?
            .map_err(|e| format!("failed to spawn command: {e}"))?;

        let mut combined = String::new();
        combined.push_str(&String::from_utf8_lossy(&output.stdout));
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            combined.push_str("\n[stderr]\n");
            combined.push_str(&stderr);
        }
        if combined.len() > MAX_OUTPUT {
            combined.truncate(MAX_OUTPUT);
            combined.push_str("\n... [truncated]");
        }
        Ok(format!(
            "exit={}\n{}",
            output.status.code().unwrap_or(-1),
            combined.trim()
        ))
    }
}
