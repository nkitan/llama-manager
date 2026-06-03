use std::path::PathBuf;

pub fn load_notes(scope: &str, base_dir: PathBuf) -> String {
    let path = if scope == "global" {
        std::path::PathBuf::from("/home/notroot/.local/share/llama-manager/global_notes.txt")
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
    let path = if scope == "global" {
        std::path::PathBuf::from("/home/notroot/.local/share/llama-manager/global_notes.txt")
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
