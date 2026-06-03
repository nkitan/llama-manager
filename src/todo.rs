use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TodoScope {
    Project,
    Global,
}

impl std::fmt::Display for TodoScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TodoScope::Project => "project",
            TodoScope::Global => "global",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Pending,
    Completed,
}

impl std::fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TodoStatus::Pending => "pending",
            TodoStatus::Completed => "completed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TodoEntry {
    pub id: String,
    pub text: String,
    pub scope: TodoScope,
    pub status: TodoStatus,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TodoList {
    pub items: Vec<TodoEntry>,
}

static TODO_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn get_todo_mutex() -> &'static Mutex<()> {
    TODO_MUTEX.get_or_init(|| Mutex::new(()))
}

impl TodoList {
    fn get_global_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
        std::path::PathBuf::from(home)
            .join(".local/share/llama-manager")
            .join("global_todos.json")
    }

    // Locked Thread-Safe API
    pub fn load_project(base_dir: PathBuf) -> Self {
        let _guard = get_todo_mutex().lock().unwrap();
        let path = base_dir.join(".llama-manager-todos.json");
        Self::load_unlocked(path)
    }

    pub fn load_global() -> Self {
        let _guard = get_todo_mutex().lock().unwrap();
        let path = Self::get_global_path();
        Self::load_unlocked(path)
    }

    pub fn save_project(&self, base_dir: PathBuf) -> Result<(), String> {
        let _guard = get_todo_mutex().lock().map_err(|e| e.to_string())?;
        let path = base_dir.join(".llama-manager-todos.json");
        self.save_unlocked(path)
    }

    pub fn save_global(&self) -> Result<(), String> {
        let _guard = get_todo_mutex().lock().map_err(|e| e.to_string())?;
        let path = Self::get_global_path();
        if let Some(p) = path.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        self.save_unlocked(path)
    }

    pub fn add_todo(entry: TodoEntry, base_dir: PathBuf) -> Result<(), String> {
        let _guard = get_todo_mutex().lock().map_err(|e| e.to_string())?;
        match entry.scope {
            TodoScope::Global => {
                let path = Self::get_global_path();
                if let Some(p) = path.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                let mut list = Self::load_unlocked(path.clone());
                list.items.push(entry);
                list.save_unlocked(path)
            }
            TodoScope::Project => {
                let path = base_dir.join(".llama-manager-todos.json");
                let mut list = Self::load_unlocked(path.clone());
                list.items.push(entry);
                list.save_unlocked(path)
            }
        }
    }

    pub fn delete_todo(id: &str, scope: TodoScope, base_dir: PathBuf) -> Result<(), String> {
        let _guard = get_todo_mutex().lock().map_err(|e| e.to_string())?;
        match scope {
            TodoScope::Global => {
                let path = Self::get_global_path();
                let mut list = Self::load_unlocked(path.clone());
                list.items.retain(|i| i.id != id);
                list.save_unlocked(path)
            }
            TodoScope::Project => {
                let path = base_dir.join(".llama-manager-todos.json");
                let mut list = Self::load_unlocked(path.clone());
                list.items.retain(|i| i.id != id);
                list.save_unlocked(path)
            }
        }
    }

    pub fn complete_todo(id: &str, scope: TodoScope, base_dir: PathBuf) -> Result<(), String> {
        let _guard = get_todo_mutex().lock().map_err(|e| e.to_string())?;
        match scope {
            TodoScope::Global => {
                let path = Self::get_global_path();
                let mut list = Self::load_unlocked(path.clone());
                if let Some(item) = list.items.iter_mut().find(|i| i.id == id) {
                    item.status = TodoStatus::Completed;
                    list.save_unlocked(path)?;
                }
            }
            TodoScope::Project => {
                let path = base_dir.join(".llama-manager-todos.json");
                let mut list = Self::load_unlocked(path.clone());
                if let Some(item) = list.items.iter_mut().find(|i| i.id == id) {
                    item.status = TodoStatus::Completed;
                    list.save_unlocked(path)?;
                }
            }
        }
        Ok(())
    }

    // Unlocked internal helper methods
    fn load_unlocked(path: PathBuf) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(list) = serde_json::from_str::<TodoList>(&content) {
                    return list;
                }
            }
        }
        TodoList::default()
    }

    fn save_unlocked(&self, path: PathBuf) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }
}
