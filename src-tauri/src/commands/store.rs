use shared::ipc::{TodoItem, Note, NotesStore};

use crate::config_io::config_dir;
use std::path::PathBuf;

fn get_notes_store_path(scope: &str) -> PathBuf {
    if scope == "global" {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
        PathBuf::from(home)
            .join(".local/share/llama-manager/global_notes.json")
    } else {
        config_dir().join(".llama-manager-notes.json")
    }
}

#[tauri::command]
pub fn notes_store_get(scope: String) -> NotesStore {
    let path = get_notes_store_path(&scope);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(store) = serde_json::from_str::<NotesStore>(&content) {
                return store;
            }
            // Migrate from old flat format
            return NotesStore {
                notes: vec![Note {
                    name: "Default".to_string(),
                    content,
                    tags: vec![],
                    pinned: false,
                }],
            };
        }
    }
    NotesStore {
        notes: vec![Note {
            name: "Default".to_string(),
            content: String::new(),
            tags: vec![],
            pinned: false,
        }],
    }
}

#[tauri::command]
pub fn notes_store_set(scope: String, store: NotesStore) -> Result<(), String> {
    let path = get_notes_store_path(&scope);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&store).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn notes_get() -> String {
    std::fs::read_to_string(config_dir().join("quick_notes.txt")).unwrap_or_default()
}

#[tauri::command]
pub fn notes_set(content: String) -> Result<(), String> {
    std::fs::write(config_dir().join("quick_notes.txt"), content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn todos_get() -> Vec<TodoItem> {
    std::fs::read_to_string(config_dir().join("agent_todos_v2.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn todos_set(items: Vec<TodoItem>) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
    std::fs::write(config_dir().join("agent_todos_v2.json"), json).map_err(|e| e.to_string())
}
