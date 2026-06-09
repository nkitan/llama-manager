//! # `run_command` — cross-platform shell executor
//!
//! Runs an arbitrary command string through the platform-native shell and
//! returns the combined output, exit code, elapsed time, and the shell used.
//!
//! ## Shell detection (automatic, no config required)
//!
//! | Platform        | Primary                              | Fallback                  |
//! |-----------------|--------------------------------------|---------------------------|
//! | **Linux / BSD** | `$SHELL` env var                     | `/bin/bash` → `/bin/sh`   |
//! | **macOS**       | `$SHELL` env var                     | `/bin/zsh` → `/bin/bash`  |
//! | **Windows**     | PowerShell 7 (`pwsh.exe`)            | PowerShell 5 (`powershell.exe`) |
//!
//! Windows PowerShell is launched with `-ExecutionPolicy Bypass -NoProfile
//! -NonInteractive -Command` so scripts and one-liners work out of the box.
//!
//! ## Safety guarantees
//! - Always requires human approval at the L4 gate before executing.
//! - Hard output cap: `16 KiB` combined stdout+stderr (truncated, never dropped).
//! - Execution ceiling: `timeout_secs` (1–120 s, default 30). Exceeded → SIGKILL.
//! - `kill_on_drop` is set — no orphan processes survive the agent task.
//! - `cwd` is validated to exist before spawning.
//!
//! ## Parameters (JSON Schema)
//!
//! | Name           | Type    | Required | Description                                        |
//! |----------------|---------|----------|----------------------------------------------------|
//! | `command`      | string  | yes      | Command string, e.g. `"ls -la"` or `"Get-Process"` |
//! | `cwd`          | string  | no       | Working directory (absolute, or relative to config dir) |
//! | `timeout_secs` | integer | no       | Max run time in seconds 1–120 (default 30)         |
//! | `env`          | object  | no       | Extra `{KEY: VALUE}` environment variable overrides |
//! | `stdin`        | string  | no       | Text fed to the command's standard input            |
//!
//! ## Example invocations
//!
//! ```json
//! { "command": "git log --oneline -10" }
//! { "command": "python3 bench.py", "cwd": "/home/user/project", "timeout_secs": 90 }
//! { "command": "npm test", "env": { "CI": "1" } }
//! { "command": "cat", "stdin": "hello world" }
//! { "command": "Get-Date; Get-Process | Select -First 5" }
//! ```
//!
//! ## Output format
//! ```
//! exit=<code> time=<ms>ms shell=<path>
//! <stdout>
//! [stderr]
//! <stderr if non-empty>
//! ```

use std::time::Duration;
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use async_trait::async_trait;
use serde_json::{Value, json};

use super::{Tool, ToolContext};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const MAX_TIMEOUT_SECS: u64 = 120;
const MAX_OUTPUT_BYTES: usize = 16_384; // 16 KiB

// ── Shell detection ──────────────────────────────────────────────────────────

/// Returns `(shell_binary, shell_args_prefix)` for the current platform.
fn resolve_shell() -> (String, Vec<String>) {
    match std::env::consts::OS {
        "windows" => {
            // Prefer PowerShell 7 (pwsh) — check common install locations without spawning.
            let ps7_exists = [
                r"C:\Program Files\PowerShell\7\pwsh.exe",
                r"C:\Program Files (x86)\PowerShell\7\pwsh.exe",
            ]
            .iter()
            .any(|p| std::path::Path::new(p).exists());

            let bin = if ps7_exists { "pwsh.exe" } else { "powershell.exe" };
            (
                bin.to_string(),
                vec![
                    "-ExecutionPolicy".into(),
                    "Bypass".into(),
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-Command".into(),
                ],
            )
        }
        "macos" => {
            // $SHELL → zsh → bash → sh
            let shell = std::env::var("SHELL")
                .ok()
                .filter(|s| !s.is_empty() && std::path::Path::new(s).exists())
                .or_else(|| {
                    std::path::Path::new("/bin/zsh")
                        .exists()
                        .then(|| "/bin/zsh".into())
                })
                .or_else(|| {
                    std::path::Path::new("/bin/bash")
                        .exists()
                        .then(|| "/bin/bash".into())
                })
                .unwrap_or_else(|| "/bin/sh".to_string());
            (shell, vec!["-c".into()])
        }
        _ => {
            // Linux, BSD, Android — $SHELL → bash → sh
            let shell = std::env::var("SHELL")
                .ok()
                .filter(|s| !s.is_empty() && std::path::Path::new(s).exists())
                .or_else(|| {
                    std::path::Path::new("/bin/bash")
                        .exists()
                        .then(|| "/bin/bash".into())
                })
                .unwrap_or_else(|| "/bin/sh".to_string());
            (shell, vec!["-c".into()])
        }
    }
}

// ── Tool implementation ──────────────────────────────────────────────────────

pub struct RunCommand;

#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &'static str {
        "run_command"
    }

    fn description(&self) -> &'static str {
        "Execute a shell command on the host machine and return its output, exit code, \
         and elapsed time. The shell is auto-detected: bash/zsh on Linux & macOS (respects \
         $SHELL), PowerShell (pwsh or powershell.exe) on Windows. \
         Optional parameters: \
         `cwd` (working directory, absolute or relative to config dir), \
         `timeout_secs` (1–120 s, default 30), \
         `env` (extra {KEY:VALUE} environment variables), \
         `stdin` (text piped to the command). \
         Output is capped at 16 KiB. Always requires user approval. \
         Examples: {\"command\":\"git status\"}, \
         {\"command\":\"python3 script.py\",\"cwd\":\"/path\",\"timeout_secs\":60}, \
         {\"command\":\"npm run build\",\"env\":{\"NODE_ENV\":\"production\"}}"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command string to run in the platform shell. \
                        On Linux/macOS this is passed to bash/zsh via -c. \
                        On Windows it is passed to PowerShell via -Command. \
                        Multi-statement commands work: \"cd /tmp && ls\" or \"Get-Date; Get-Process\"."
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for the command. \
                        Absolute paths are used as-is. Relative paths are resolved \
                        from the application config directory. Omit to use config dir."
                },
                "timeout_secs": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 120,
                    "default": 30,
                    "description": "Maximum execution time in seconds (1–120). \
                        The process is killed after this limit. Default: 30."
                },
                "env": {
                    "type": "object",
                    "description": "Additional environment variables to set for this command only. \
                        Example: {\"DEBUG\": \"1\", \"MY_TOKEN\": \"abc\"}. \
                        These are merged with (not replace) the inherited environment.",
                    "additionalProperties": { "type": "string" }
                },
                "stdin": {
                    "type": "string",
                    "description": "Optional text to pipe to the command's standard input. \
                        Useful for commands that read from stdin, e.g. {\"command\":\"cat\", \"stdin\":\"hello\"}."
                }
            },
            "required": ["command"]
        })
    }

    fn is_sensitive(&self) -> bool {
        true
    }

    async fn run(&self, args: &Value, ctx: &ToolContext) -> Result<String, String> {
        // ── Validate inputs ──────────────────────────────────────────────────
        let command_str = args["command"].as_str().unwrap_or("").trim();
        if command_str.is_empty() {
            return Err("`command` is required and must be a non-empty string.".into());
        }

        // ── Resolve working directory ────────────────────────────────────────
        let cwd = match args["cwd"].as_str().filter(|s| !s.is_empty()) {
            Some(raw) => {
                let p = std::path::Path::new(raw);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    ctx.config_dir.join(raw)
                }
            }
            None => ctx.config_dir.clone(),
        };
        if !cwd.exists() {
            return Err(format!(
                "Working directory does not exist: `{}`. \
                 Use an absolute path or a path relative to the config directory.",
                cwd.display()
            ));
        }

        // ── Timeout (clamp to 1–120 s) ───────────────────────────────────────
        let timeout_secs = args["timeout_secs"]
            .as_u64()
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .clamp(1, MAX_TIMEOUT_SECS);

        // ── Extra environment variables ──────────────────────────────────────
        let extra_env: Vec<(String, String)> = args["env"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        // ── Optional stdin ────────────────────────────────────────────────────
        let stdin_data: Option<String> = args["stdin"].as_str().map(|s| s.to_string());

        // ── Detect shell ──────────────────────────────────────────────────────
        let (shell_bin, shell_prefix) = resolve_shell();

        // ── Build and spawn the process ───────────────────────────────────────
        let mut cmd = Command::new(&shell_bin);
        cmd.args(&shell_prefix)
            .arg(command_str)
            .current_dir(&cwd)
            .envs(extra_env)
            .stdin(if stdin_data.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let start = std::time::Instant::now();

        let mut child = cmd.spawn().map_err(|e| {
            format!(
                "Failed to spawn shell `{shell_bin}`: {e}. \
                 Check that the shell is installed and accessible."
            )
        })?;

        // Write stdin before waiting (drop closes the pipe → EOF to child)
        if let Some(input) = stdin_data {
            if let Some(mut stdin_handle) = child.stdin.take() {
                let _ = stdin_handle.write_all(input.as_bytes()).await;
                // explicit drop ensures EOF is sent before we wait
                drop(stdin_handle);
            }
        }

        // ── Wait with timeout ─────────────────────────────────────────────────
        let output = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| {
            format!(
                "Command killed: timed out after {timeout_secs}s. \
                 Increase `timeout_secs` (max 120) or break the task into smaller commands."
            )
        })?
        .map_err(|e| format!("Command execution failed: {e}"))?;

        let elapsed_ms = start.elapsed().as_millis();
        let exit_code = output.status.code().unwrap_or(-1);

        // ── Format output ─────────────────────────────────────────────────────
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = format!(
            "exit={exit_code} time={elapsed_ms}ms shell={shell_bin}\n"
        );

        if !stdout.trim().is_empty() {
            result.push_str(stdout.trim_end());
            result.push('\n');
        }
        if !stderr.trim().is_empty() {
            result.push_str("[stderr]\n");
            result.push_str(stderr.trim_end());
            result.push('\n');
        }
        if stdout.trim().is_empty() && stderr.trim().is_empty() {
            result.push_str("(no output)");
        }

        // Cap at 16 KiB
        if result.len() > MAX_OUTPUT_BYTES {
            result.truncate(MAX_OUTPUT_BYTES);
            result.push_str("\n... [output truncated at 16 KiB]");
        }

        Ok(result.trim_end().to_string())
    }
}
