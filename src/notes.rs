use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static NOTES_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn get_notes_mutex() -> &'static Mutex<()> {
    NOTES_MUTEX.get_or_init(|| Mutex::new(()))
}

pub fn load_notes(scope: &str, base_dir: PathBuf) -> String {
    let _guard = get_notes_mutex().lock().unwrap();
    let path = if scope == "global" {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
        std::path::PathBuf::from(home)
            .join(".local/share/llama-manager/global_notes.txt")
    } else {
        base_dir.join(".llama-manager-notes.txt")
    };

    if path.exists() {
        std::fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    }
}

pub fn save_notes(scope: &str, content: &str, base_dir: PathBuf) -> Result<(), String> {
    let _guard = get_notes_mutex().lock().map_err(|e| e.to_string())?;
    let path = if scope == "global" {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
        std::path::PathBuf::from(home)
            .join(".local/share/llama-manager/global_notes.txt")
    } else {
        base_dir.join(".llama-manager-notes.txt")
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write notes: {}", e))?;
    Ok(())
}
