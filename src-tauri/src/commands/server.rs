//! llama-server lifecycle + native path picking.
//!
//! `server_start` spawns the process from the canonical config's `to_args()`,
//! streaming each stdout/stderr line to the UI as a `server://log` event (the
//! same streaming pattern as chat/agent). The child handle lives in `AppState`
//! so `server_stop`/`server_status` can manage it. Persistent tracking is
//! maintained via a PID file in the config directory.

use std::process::Stdio;

use shared::ipc::SERVER_LOG_EVENT;
use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::state::AppState;

fn get_pid_if_running() -> Option<u32> {
    let pid_path = crate::config_io::config_dir().join("llama-server.pid");
    if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let proc_dir = format!("/proc/{}", pid);
            if std::path::Path::new(&proc_dir).exists() {
                let comm_path = format!("/proc/{}/comm", pid);
                if let Ok(comm) = std::fs::read_to_string(&comm_path) {
                    let comm_clean = comm.trim().to_lowercase();
                    if comm_clean.contains("llama") || comm_clean.contains("server") {
                        return Some(pid);
                    }
                } else {
                    return Some(pid);
                }
            }
        }
    }
    None
}

async fn kill_pid(pid: u32) {
    let pid_str = pid.to_string();
    let _ = std::process::Command::new("kill").arg(&pid_str).status();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let proc_dir = format!("/proc/{}", pid);
    if std::path::Path::new(&proc_dir).exists() {
        let _ = std::process::Command::new("kill").arg("-9").arg(&pid_str).status();
    }
}

#[tauri::command]
pub async fn server_start(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if state.server.lock().unwrap().is_some() || get_pid_if_running().is_some() {
        return Err("Server is already running.".into());
    }
    let cfg = state.config.lock().unwrap().clone();
    if cfg.model_path.is_empty()
        && cfg.hf_repo.is_empty()
        && cfg.model_url.is_empty()
        && cfg.model_dir.is_empty()
    {
        return Err("Provide a model path, HuggingFace repo, model dir, or URL first.".into());
    }

    let exe = cfg.exe_path.clone();
    let args = cfg.to_args();
    let _ = app.emit(
        SERVER_LOG_EVENT,
        format!("[MANAGER] Starting: {} {}", exe, args.join(" ")),
    );
    tracing::info!(%exe, "server_start");

    let mut child = Command::new(&exe)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start `{exe}`: {e}"))?;

    // Stream stdout + stderr line-by-line to the UI.
    for pipe in [child.stdout.take().map(Pipe::Out), child.stderr.take().map(Pipe::Err)]
        .into_iter()
        .flatten()
    {
        let app = app.clone();
        tokio::spawn(async move {
            match pipe {
                Pipe::Out(o) => stream_lines(app, BufReader::new(o)).await,
                Pipe::Err(e) => stream_lines(app, BufReader::new(e)).await,
            }
        });
    }

    if let Some(pid) = child.id() {
        let pid_path = crate::config_io::config_dir().join("llama-server.pid");
        let _ = std::fs::write(&pid_path, pid.to_string());
    }

    *state.server.lock().unwrap() = Some(child);
    Ok(())
}

enum Pipe {
    Out(tokio::process::ChildStdout),
    Err(tokio::process::ChildStderr),
}

async fn stream_lines<R: tokio::io::AsyncRead + Unpin>(app: AppHandle, reader: BufReader<R>) {
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let _ = app.emit(SERVER_LOG_EVENT, line);
    }
}

#[tauri::command]
pub async fn server_stop(state: State<'_, AppState>) -> Result<(), String> {
    let child = state.server.lock().unwrap().take();
    let pid_path = crate::config_io::config_dir().join("llama-server.pid");
    match child {
        Some(mut c) => {
            c.kill().await.map_err(|e| e.to_string())?;
            let _ = std::fs::remove_file(pid_path);
            
            // Reap the child process so it's fully gone
            let _ = c.wait().await;
            
            // Sleep briefly to allow port release
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            
            tracing::info!("server_stop (in-memory child)");
            Ok(())
        }
        None => {
            if let Some(pid) = get_pid_if_running() {
                kill_pid(pid).await;
                let _ = std::fs::remove_file(pid_path);
                
                // Sleep briefly to allow port release
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                
                tracing::info!(%pid, "server_stop (PID file process killed)");
                Ok(())
            } else {
                let _ = std::fs::remove_file(pid_path);
                Err("Server is not running.".into())
            }
        }
    }
}

/// True if a child is tracked and still alive (reaps it if it has exited).
#[tauri::command]
pub fn server_status(state: State<'_, AppState>) -> bool {
    let mut guard = state.server.lock().unwrap();
    let pid_path = crate::config_io::config_dir().join("llama-server.pid");
    if let Some(child) = guard.as_mut() {
        match child.try_wait() {
            Ok(Some(_)) => {
                *guard = None;
                let _ = std::fs::remove_file(pid_path);
                false
            }
            _ => true,
        }
    } else {
        if get_pid_if_running().is_some() {
            true
        } else {
            let _ = std::fs::remove_file(pid_path);
            false
        }
    }
}

/// Native file/folder picker. Returns the chosen path, or `None` if cancelled.
#[tauri::command]
pub async fn pick_path(directory: bool) -> Option<String> {
    tokio::task::spawn_blocking(move || {
        let dialog = rfd::FileDialog::new();
        if directory {
            dialog.pick_folder()
        } else {
            dialog.pick_file()
        }
        .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .ok()
    .flatten()
}
