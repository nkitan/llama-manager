use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub time: String,      // Format: "YYYY-MM-DDTHH:MM:SS"
    pub prompt: String,    // Prompt to fire when the event time is reached
    pub status: String,    // "pending", "firing", "completed", "failed"
    pub result: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CalendarState {
    pub events: Vec<CalendarEvent>,
}

impl CalendarState {
    pub fn load() -> Self {
        let path = Self::get_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<CalendarState>(&content) {
                    return state;
                }
            }
        }
        CalendarState::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_path();
        if let Some(p) = path.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }

    fn get_path() -> PathBuf {
        std::path::PathBuf::from("/home/notroot/.local/share/llama-manager/calendar.json")
    }
}
