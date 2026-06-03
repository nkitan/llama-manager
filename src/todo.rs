use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TodoEntry {
    pub id: String,
    pub text: String,
    pub scope: String,      // "project" or "global"
    pub status: String,     // "pending" or "completed"
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TodoList {
    pub items: Vec<TodoEntry>,
}

impl TodoList {
    pub fn load_project(base_dir: PathBuf) -> Self {
        let path = base_dir.join(".llama-manager-todos.json");
        Self::load(path)
    }

    pub fn load_global() -> Self {
        let path = std::path::PathBuf::from("/home/notroot/.local/share/llama-manager/global_todos.json");
        Self::load(path)
    }

    fn load(path: PathBuf) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(list) = serde_json::from_str::<TodoList>(&content) {
                    return list;
                }
            }
        }
        TodoList::default()
    }

    pub fn save_project(&self, base_dir: PathBuf) -> Result<(), String> {
        let path = base_dir.join(".llama-manager-todos.json");
        self.save(path)
    }

    pub fn save_global(&self) -> Result<(), String> {
        let path = std::path::PathBuf::from("/home/notroot/.local/share/llama-manager/global_todos.json");
        if let Some(p) = path.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        self.save(path)
    }

    fn save(&self, path: PathBuf) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }
}
