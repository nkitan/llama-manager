use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Backlog => "Backlog",
            Self::Todo => "Todo",
            Self::InProgress => "In Progress",
            Self::Done => "Done",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KanbanTask {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub assigned_agent: String,
    pub status: TaskStatus,
    pub priority: String, // "Low", "Medium", "High"
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlannerState {
    pub tasks: Vec<KanbanTask>,
}

impl PlannerState {
    pub fn load(path: PathBuf) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<PlannerState>(&content) {
                    return state;
                }
            }
        }
        // Provide default items on first boot
        let mut default_state = PlannerState::default();
        default_state.tasks.push(KanbanTask {
            id: "task-1".to_string(),
            title: "Optimize Llama Server Parameters".to_string(),
            summary: "Tune micro-batch size and GPU layers for hardware acceleration".to_string(),
            assigned_agent: "ConfigOptimizer".to_string(),
            status: TaskStatus::Todo,
            priority: "High".to_string(),
            created_at: "2026-06-03".to_string(),
        });
        default_state.tasks.push(KanbanTask {
            id: "task-2".to_string(),
            title: "Internet Research on Hermes-Agent".to_string(),
            summary: "Search repositories and documentation for core implementation examples"
                .to_string(),
            assigned_agent: "WebResearcher".to_string(),
            status: TaskStatus::InProgress,
            priority: "Medium".to_string(),
            created_at: "2026-06-03".to_string(),
        });
        let _ = default_state.save(path);
        default_state
    }

    pub fn save(&self, path: PathBuf) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }
}

// ── Agent Activity and Status Monitoring ─────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentStatus {
    pub name: String,
    pub current_action: String,
    pub status: String, // "Idle", "Active", "Offline"
    pub last_active: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentEvent {
    pub timestamp: String,
    pub agent_name: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MonitorState {
    pub agents: Vec<AgentStatus>,
    pub events: Vec<AgentEvent>,
}

impl MonitorState {
    pub fn load(path: PathBuf) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<MonitorState>(&content) {
                    return state;
                }
            }
        }
        // Default initial monitoring state
        let mut default_state = MonitorState::default();
        default_state.agents.push(AgentStatus {
            name: "ConfigOptimizer".to_string(),
            current_action: "Idle".to_string(),
            status: "Idle".to_string(),
            last_active: "Just now".to_string(),
        });
        default_state.agents.push(AgentStatus {
            name: "WebResearcher".to_string(),
            current_action: "Scraping LLM specifications".to_string(),
            status: "Active".to_string(),
            last_active: "Just now".to_string(),
        });
        default_state.events.push(AgentEvent {
            timestamp: "22:20:10".to_string(),
            agent_name: "ConfigOptimizer".to_string(),
            message: "Finished analyzing optimization profiles".to_string(),
        });
        default_state.events.push(AgentEvent {
            timestamp: "22:23:01".to_string(),
            agent_name: "WebResearcher".to_string(),
            message: "Started DuckDuckGo query: 'hermes-agent python codebase'".to_string(),
        });
        let _ = default_state.save(path);
        default_state
    }

    pub fn save(&self, path: PathBuf) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("IO error: {}", e))?;
        Ok(())
    }

    pub fn add_event(&mut self, agent: &str, msg: &str, path: PathBuf) {
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        self.events.push(AgentEvent {
            timestamp: now,
            agent_name: agent.to_string(),
            message: msg.to_string(),
        });
        // Update agent action
        if let Some(a) = self.agents.iter_mut().find(|a| a.name == agent) {
            a.current_action = msg.to_string();
            a.status = "Active".to_string();
            a.last_active = "Just now".to_string();
        } else {
            self.agents.push(AgentStatus {
                name: agent.to_string(),
                current_action: msg.to_string(),
                status: "Active".to_string(),
                last_active: "Just now".to_string(),
            });
        }
        // Limit events list size
        if self.events.len() > 100 {
            self.events.remove(0);
        }
        let _ = self.save(path);
    }

    pub fn set_idle(&mut self, agent: &str, path: PathBuf) {
        if let Some(a) = self.agents.iter_mut().find(|a| a.name == agent) {
            a.current_action = "Idle".to_string();
            a.status = "Idle".to_string();
        }
        let _ = self.save(path);
    }
}
