use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Pending,
    Firing,
    Completed,
    Failed,
}

impl std::fmt::Display for EventStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EventStatus::Pending => "pending",
            EventStatus::Firing => "firing",
            EventStatus::Completed => "completed",
            EventStatus::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub time: String,      // Format: "YYYY-MM-DDTHH:MM:SS"
    pub prompt: String,    // Prompt to fire when the event time is reached
    pub status: EventStatus,
    pub result: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CalendarState {
    pub events: Vec<CalendarEvent>,
}

static CALENDAR_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn get_calendar_mutex() -> &'static Mutex<()> {
    CALENDAR_MUTEX.get_or_init(|| Mutex::new(()))
}

pub fn parse_datetime(s: &str) -> Result<chrono::NaiveDateTime, String> {
    let s = s.trim();
    let formats = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
    ];
    for fmt in &formats {
        if *fmt == "%Y-%m-%d" {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
                if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                    return Ok(dt);
                }
            }
        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(dt);
        }
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.naive_local());
    }
    Err(format!("Invalid datetime format: '{}'", s))
}

impl CalendarState {
    // Locked Thread-Safe API
    pub fn load() -> Self {
        let _guard = get_calendar_mutex().lock().unwrap();
        Self::load_unlocked()
    }

    pub fn save(&self) -> Result<(), String> {
        let _guard = get_calendar_mutex().lock().map_err(|e| e.to_string())?;
        self.save_unlocked()
    }

    pub fn add_event(ev: CalendarEvent) -> Result<(), String> {
        let _guard = get_calendar_mutex().lock().map_err(|e| e.to_string())?;
        let mut state = Self::load_unlocked();
        state.events.push(ev);
        state.save_unlocked()
    }

    pub fn delete_event(id: &str) -> Result<(), String> {
        let _guard = get_calendar_mutex().lock().map_err(|e| e.to_string())?;
        let mut state = Self::load_unlocked();
        state.events.retain(|e| e.id != id);
        state.save_unlocked()
    }

    pub fn update_event<F>(id: &str, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut CalendarEvent),
    {
        let _guard = get_calendar_mutex().lock().map_err(|e| e.to_string())?;
        let mut state = Self::load_unlocked();
        if let Some(ev) = state.events.iter_mut().find(|e| e.id == id) {
            f(ev);
            state.save_unlocked()?;
        }
        Ok(())
    }

    pub fn checkout_firing_events(now: &str) -> Vec<CalendarEvent> {
        let _guard = get_calendar_mutex().lock().unwrap();
        let mut state = Self::load_unlocked();
        let mut to_fire = Vec::new();
        let now_dt = parse_datetime(now).unwrap_or_else(|_| chrono::Local::now().naive_local());
        
        for ev in state.events.iter_mut() {
            if ev.status == EventStatus::Pending {
                if let Ok(ev_dt) = parse_datetime(&ev.time) {
                    if ev_dt <= now_dt {
                        ev.status = EventStatus::Firing;
                        to_fire.push(ev.clone());
                    }
                } else if ev.time.as_str() <= now {
                    // Fallback to string comparison
                    ev.status = EventStatus::Firing;
                    to_fire.push(ev.clone());
                }
            }
        }
        if !to_fire.is_empty() {
            let _ = state.save_unlocked();
        }
        to_fire
    }


    // Unlocked internal helper methods
    fn load_unlocked() -> Self {
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

    fn save_unlocked(&self) -> Result<(), String> {
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
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
        std::path::PathBuf::from(home)
            .join(".local/share/llama-manager")
            .join("calendar.json")
    }
}
