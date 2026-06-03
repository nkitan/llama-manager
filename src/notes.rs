use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static NOTES_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn get_notes_mutex() -> &'static Mutex<()> {
    NOTES_MUTEX.get_or_init(|| Mutex::new(()))
}

/// A single note with a name and content.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Note {
    pub name: String,
    pub content: String,
}

/// The on-disk format for storing multiple notes in one scope.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NotesStore {
    pub notes: Vec<Note>,
}

impl NotesStore {
    pub fn load(scope: &str, base_dir: PathBuf) -> Self {
        let _guard = get_notes_mutex().lock().unwrap();
        let path = get_store_path(scope, &base_dir);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Try to load new JSON format
                if let Ok(store) = serde_json::from_str::<NotesStore>(&content) {
                    return store;
                }
                // Migrate from old single-note text format
                return NotesStore {
                    notes: vec![Note {
                        name: "Default".to_string(),
                        content,
                    }],
                };
            }
        }
        // No file yet – start with one empty note
        NotesStore {
            notes: vec![Note {
                name: "Default".to_string(),
                content: String::new(),
            }],
        }
    }

    pub fn save(&self, scope: &str, base_dir: PathBuf) -> Result<(), String> {
        let _guard = get_notes_mutex().lock().map_err(|e| e.to_string())?;
        let path = get_store_path(scope, &base_dir);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(&path, json).map_err(|e| format!("IO error: {}", e))
    }

    /// Add a new note (returns index of new note).
    pub fn add_note(&mut self, name: String) -> usize {
        self.notes.push(Note {
            name,
            content: String::new(),
        });
        self.notes.len() - 1
    }

    /// Delete a note by index, keeping at least one.
    pub fn delete_note(&mut self, idx: usize) {
        if self.notes.len() > 1 && idx < self.notes.len() {
            self.notes.remove(idx);
        } else if self.notes.len() == 1 {
            self.notes[0].content = String::new();
        }
    }

    /// Rename a note by index.
    pub fn rename_note(&mut self, idx: usize, new_name: String) {
        if idx < self.notes.len() {
            self.notes[idx].name = new_name;
        }
    }
}

fn get_store_path(scope: &str, base_dir: &PathBuf) -> PathBuf {
    if scope == "global" {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
        std::path::PathBuf::from(home)
            .join(".local/share/llama-manager/global_notes.json")
    } else {
        base_dir.join(".llama-manager-notes.json")
    }
}

// ── Legacy single-note API (kept for agent compatibility) ────────────────────

pub fn load_notes(scope: &str, base_dir: PathBuf) -> String {
    let store = NotesStore::load(scope, base_dir);
    store.notes.first().map(|n| n.content.clone()).unwrap_or_default()
}

pub fn save_notes(scope: &str, content: &str, base_dir: PathBuf) -> Result<(), String> {
    let mut store = NotesStore::load(scope, base_dir.clone());
    if let Some(first) = store.notes.first_mut() {
        first.content = content.to_string();
    } else {
        store.notes.push(Note {
            name: "Default".to_string(),
            content: content.to_string(),
        });
    }
    store.save(scope, base_dir)
}
