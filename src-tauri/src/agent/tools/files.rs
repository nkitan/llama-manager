//! File-backed productivity tools for reading and writing Todos and Notes.
//! These write directly to the same files used by the UI components.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::PathBuf;
use shared::ipc::{TodoItem, Note, NotesStore};
use crate::config_io::config_dir;

use super::{Tool, ToolContext};

fn get_notes_store_path(scope: &str) -> PathBuf {
    if scope == "global" {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/notroot".to_string());
        PathBuf::from(home)
            .join(".local/share/llama-manager/global_notes.json")
    } else {
        config_dir().join(".llama-manager-notes.json")
    }
}

fn load_notes_store(scope: &str) -> NotesStore {
    let path = get_notes_store_path(scope);
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
                }],
            };
        }
    }
    NotesStore {
        notes: vec![Note {
            name: "Default".to_string(),
            content: String::new(),
        }],
    }
}

fn save_notes_store(scope: &str, store: &NotesStore) -> Result<(), String> {
    let path = get_notes_store_path(scope);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_todos() -> Vec<TodoItem> {
    std::fs::read_to_string(config_dir().join("agent_todos_v2.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_todos(items: &[TodoItem]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(items).map_err(|e| e.to_string())?;
    std::fs::write(config_dir().join("agent_todos_v2.json"), json).map_err(|e| e.to_string())
}

// ── GET TODOS TOOL ───────────────────────────────────────────────────────────

pub struct GetTodos;

#[async_trait]
impl Tool for GetTodos {
    fn name(&self) -> &'static str {
        "get_todos"
    }
    fn description(&self) -> &'static str {
        "Read all todo items in the user's checklist. Returns a list of todos with their IDs, text, and completion status."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }
    async fn run(&self, _args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let items = load_todos();
        if items.is_empty() {
            Ok("No todo items in checklist.".to_string())
        } else {
            let res = items.iter()
                .map(|item| format!("- [{}] (id: {}): {}", if item.done { "x" } else { " " }, item.id, item.text))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(res)
        }
    }
}

// ── ADD TODO TOOL ────────────────────────────────────────────────────────────

pub struct AddTodo;

#[async_trait]
impl Tool for AddTodo {
    fn name(&self) -> &'static str {
        "add_todo"
    }
    fn description(&self) -> &'static str {
        "Append a todo item to the user's checklist. Mutates user data."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "The text content of the todo item to add." }
            },
            "required": ["text"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let text = args["text"].as_str().unwrap_or("").trim().to_string();
        if text.is_empty() {
            return Err("`text` is required".into());
        }
        let mut items = load_todos();
        let id = format!("t{}", chrono::Utc::now().timestamp_millis());
        items.push(TodoItem { id: id.clone(), text: text.clone(), done: false });
        save_todos(&items)?;
        Ok(format!("Added todo: {} (id: {})", text, id))
    }
}

// ── COMPLETE TODO TOOL ────────────────────────────────────────────────────────

pub struct CompleteTodo;

#[async_trait]
impl Tool for CompleteTodo {
    fn name(&self) -> &'static str {
        "complete_todo"
    }
    fn description(&self) -> &'static str {
        "Mark a todo item as completed, uncompleted, or delete it from the checklist."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "The unique ID of the todo item (e.g. 't171228...')." },
                "done": { "type": "boolean", "description": "Whether the todo item should be marked as completed (true) or uncompleted (false)." },
                "delete": { "type": "boolean", "description": "Optional. Set to true to delete the todo item completely." }
            },
            "required": ["id"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let id = args["id"].as_str().unwrap_or("").trim().to_string();
        if id.is_empty() {
            return Err("`id` is required".into());
        }
        let done = args["done"].as_bool().unwrap_or(true);
        let delete = args["delete"].as_bool().unwrap_or(false);

        let mut items = load_todos();
        let exists = items.iter().any(|item| item.id == id);
        if !exists {
            return Err(format!("No todo item found with id '{}'. Use get_todos to check IDs.", id));
        }

        if delete {
            items.retain(|item| item.id != id);
            save_todos(&items)?;
            Ok(format!("Deleted todo item '{}'.", id))
        } else {
            for item in &mut items {
                if item.id == id {
                    item.done = done;
                }
            }
            save_todos(&items)?;
            Ok(format!("Marked todo item '{}' as {}.", id, if done { "completed" } else { "pending" }))
        }
    }
}

// ── READ NOTES TOOL ──────────────────────────────────────────────────────────

pub struct ReadNotes;

#[async_trait]
impl Tool for ReadNotes {
    fn name(&self) -> &'static str {
        "read_notes"
    }
    fn description(&self) -> &'static str {
        "Read the user's quick notes. Can specify scope (project or global)."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "string",
                    "enum": ["project", "global"],
                    "default": "project",
                    "description": "The scope of notes to read. 'project' reads project-specific notes; 'global' reads global user notes."
                }
            }
        })
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let scope = args["scope"].as_str().unwrap_or("project");
        let store = load_notes_store(scope);
        if store.notes.is_empty() {
            Ok(format!("No notes found in scope '{}'.", scope))
        } else {
            let mut res = format!("--- Quick Notes ({}) ---\n", scope);
            for note in &store.notes {
                res.push_str(&format!("# {}\n{}\n\n", note.name, note.content));
            }
            Ok(res)
        }
    }
}

// ── WRITE NOTE TOOL ──────────────────────────────────────────────────────────

pub struct WriteNote;

#[async_trait]
impl Tool for WriteNote {
    fn name(&self) -> &'static str {
        "write_note"
    }
    fn description(&self) -> &'static str {
        "Create, update, or append to a quick note in the user's notes store."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "string",
                    "enum": ["project", "global"],
                    "default": "project",
                    "description": "The scope of the note to write."
                },
                "name": {
                    "type": "string",
                    "description": "The title/name of the note to create or update (e.g. 'Setup Instructions')."
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the note."
                },
                "append": {
                    "type": "boolean",
                    "description": "Optional. If true, appends the content to the existing note content instead of overwriting."
                }
            },
            "required": ["name", "content"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let scope = args["scope"].as_str().unwrap_or("project");
        let name = args["name"].as_str().unwrap_or("").trim().to_string();
        let content = args["content"].as_str().unwrap_or("").to_string();
        let append = args["append"].as_bool().unwrap_or(false);

        if name.is_empty() {
            return Err("`name` is required".into());
        }

        let mut store = load_notes_store(scope);
        let mut found = false;

        for note in &mut store.notes {
            if note.name.to_lowercase() == name.to_lowercase() {
                if append {
                    note.content.push_str("\n");
                    note.content.push_str(&content);
                } else {
                    note.content = content.clone();
                }
                found = true;
                break;
            }
        }

        if !found {
            store.notes.push(Note {
                name: name.clone(),
                content: content.clone(),
            });
        }

        save_notes_store(scope, &store)?;
        Ok(format!("Successfully wrote to note '{}' in scope '{}'.", name, scope))
    }
}

// ── CALENDAR & PLANNER INTEGRATION FOR AGENTS ──

use shared::ipc::{CalendarEvent, CalendarState, KanbanTask, PlannerState, TaskStatus, EventStatus};

fn load_calendar() -> CalendarState {
    let path = config_dir().join("calendar.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<CalendarState>(&content) {
                return state;
            }
        }
    }
    CalendarState::default()
}

fn save_calendar(state: &CalendarState) -> Result<(), String> {
    let path = config_dir().join("calendar.json");
    let content = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

fn load_planner() -> PlannerState {
    let path = config_dir().join("planner_tasks.json");
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<PlannerState>(&content) {
                return state;
            }
        }
    }
    PlannerState::default()
}

fn save_planner(state: &PlannerState) -> Result<(), String> {
    let path = config_dir().join("planner_tasks.json");
    let content = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

// ── GET CALENDAR EVENTS TOOL ───────────────────────────────────────────────

pub struct GetCalendarEvents;

#[async_trait]
impl Tool for GetCalendarEvents {
    fn name(&self) -> &'static str {
        "get_calendar_events"
    }
    fn description(&self) -> &'static str {
        "Read all scheduled events in the user's calendar/scheduler. Returns title, time, prompt trigger, notes, and execution status."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }
    async fn run(&self, _args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let state = load_calendar();
        if state.events.is_empty() {
            Ok("No scheduled events in calendar.".to_string())
        } else {
            let mut res = String::from("--- Calendar Scheduled Events ---\n");
            for ev in &state.events {
                res.push_str(&format!(
                    "- [{:?}] ID: {}, Title: '{}', Time: {}, Trigger Prompt: '{}'\n  Notes: {}\n",
                    ev.status,
                    ev.id,
                    ev.title,
                    ev.time,
                    ev.prompt,
                    ev.note.as_deref().unwrap_or("(none)")
                ));
            }
            Ok(res)
        }
    }
}

// ── ADD CALENDAR EVENT TOOL ────────────────────────────────────────────────

pub struct AddCalendarEvent;

#[async_trait]
impl Tool for AddCalendarEvent {
    fn name(&self) -> &'static str {
        "add_calendar_event"
    }
    fn description(&self) -> &'static str {
        "Add a new scheduled automation prompt/event to the user's calendar. Format date as YYYY-MM-DDTHH:MM:SS."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "The title of the calendar event." },
                "time": { "type": "string", "description": "Date and time formatted as YYYY-MM-DDTHH:MM:SS, e.g. '2026-06-04T12:00:00'." },
                "prompt": { "type": "string", "description": "The prompt trigger to execute when the scheduled time is reached." },
                "note": { "type": "string", "description": "Optional notes/context for the scheduled event." },
                "color": { "type": "string", "description": "Optional UI display color hex code or name, e.g. '#3b82f6'." }
            },
            "required": ["title", "time", "prompt"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let title = args["title"].as_str().unwrap_or("").trim().to_string();
        let time = args["time"].as_str().unwrap_or("").trim().to_string();
        let prompt = args["prompt"].as_str().unwrap_or("").trim().to_string();
        let note = args["note"].as_str().map(|s| s.trim().to_string());
        let color = args["color"].as_str().map(|s| s.trim().to_string());

        if title.is_empty() || time.is_empty() || prompt.is_empty() {
            return Err("`title`, `time`, and `prompt` are required fields".into());
        }

        let mut state = load_calendar();
        let id = format!("e{}", chrono::Utc::now().timestamp_millis());
        let new_event = CalendarEvent {
            id: id.clone(),
            title: title.clone(),
            time,
            prompt,
            note,
            color,
            status: EventStatus::Pending,
            result: None,
        };
        state.events.push(new_event);
        save_calendar(&state)?;
        Ok(format!("Successfully scheduled event: '{}' (id: {})", title, id))
    }
}

// ── DELETE CALENDAR EVENT TOOL ─────────────────────────────────────────────

pub struct DeleteCalendarEvent;

#[async_trait]
impl Tool for DeleteCalendarEvent {
    fn name(&self) -> &'static str {
        "delete_calendar_event"
    }
    fn description(&self) -> &'static str {
        "Remove a scheduled event/automation trigger from the calendar using its unique ID."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "The unique ID of the event to delete (e.g. 'e171228...')." }
            },
            "required": ["id"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let id = args["id"].as_str().unwrap_or("").trim().to_string();
        if id.is_empty() {
            return Err("`id` is required".into());
        }

        let mut state = load_calendar();
        let len_before = state.events.len();
        state.events.retain(|e| e.id != id);

        if state.events.len() == len_before {
            return Err(format!("No scheduled event found with ID '{}'.", id));
        }

        save_calendar(&state)?;
        Ok(format!("Successfully deleted scheduled event '{}'.", id))
    }
}

// ── GET PLANNER TASKS TOOL ─────────────────────────────────────────────────

pub struct GetPlannerTasks;

#[async_trait]
impl Tool for GetPlannerTasks {
    fn name(&self) -> &'static str {
        "get_planner_tasks"
    }
    fn description(&self) -> &'static str {
        "Read all tasks and backlog items on the user's task planner/Kanban board, detailing assignees, priorities, and status."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }
    async fn run(&self, _args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let state = load_planner();
        if state.tasks.is_empty() {
            Ok("No tasks found in planner.".to_string())
        } else {
            let mut res = String::from("--- Planner Tasks ---\n");
            for t in &state.tasks {
                res.push_str(&format!(
                    "- [{:?}] ID: {}, Title: '{}'\n  Summary: {}\n  Assignee: {}, Priority: {}, Created: {}\n",
                    t.status,
                    t.id,
                    t.title,
                    t.summary,
                    t.assigned_agent,
                    t.priority,
                    t.created_at
                ));
            }
            Ok(res)
        }
    }
}

// ── ADD PLANNER TASK TOOL ──────────────────────────────────────────────────

pub struct AddPlannerTask;

#[async_trait]
impl Tool for AddPlannerTask {
    fn name(&self) -> &'static str {
        "add_planner_task"
    }
    fn description(&self) -> &'static str {
        "Create and insert a new task card onto the task planner/Kanban board."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "The concise title of the task." },
                "summary": { "type": "string", "description": "A detailed description or steps for this task." },
                "assigned_agent": { "type": "string", "description": "The agent/person assigned to the task (e.g. 'WebResearcher' or 'Developer')." },
                "priority": { "type": "string", "enum": ["Low", "Medium", "High"], "default": "Medium", "description": "Task priority level." },
                "status": { "type": "string", "enum": ["Backlog", "Todo", "In Progress", "Done"], "default": "Todo", "description": "Initial column placement." }
            },
            "required": ["title", "summary"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let title = args["title"].as_str().unwrap_or("").trim().to_string();
        let summary = args["summary"].as_str().unwrap_or("").trim().to_string();
        let assigned_agent = args["assigned_agent"].as_str().unwrap_or("ConfigOptimizer").trim().to_string();
        let priority = args["priority"].as_str().unwrap_or("Medium").to_string();
        let status_str = args["status"].as_str().unwrap_or("Todo");

        if title.is_empty() || summary.is_empty() {
            return Err("`title` and `summary` are required".into());
        }

        let status = match status_str {
            "Backlog" => TaskStatus::Backlog,
            "In Progress" => TaskStatus::InProgress,
            "Done" => TaskStatus::Done,
            _ => TaskStatus::Todo,
        };

        let mut state = load_planner();
        let id = format!("task-{}", chrono::Utc::now().timestamp_millis());
        let created_at = chrono::Local::now().format("%Y-%m-%d").to_string();

        let new_task = KanbanTask {
            id: id.clone(),
            title: title.clone(),
            summary,
            assigned_agent,
            status,
            priority,
            created_at,
        };
        state.tasks.push(new_task);
        save_planner(&state)?;
        Ok(format!("Successfully created planner task: '{}' (id: {})", title, id))
    }
}

// ── UPDATE PLANNER TASK TOOL ───────────────────────────────────────────────

pub struct UpdatePlannerTask;

#[async_trait]
impl Tool for UpdatePlannerTask {
    fn name(&self) -> &'static str {
        "update_planner_task"
    }
    fn description(&self) -> &'static str {
        "Modify properties or move columns of a planner task card by its ID."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "The unique ID of the task to update, e.g. 'task-1'." },
                "status": { "type": "string", "enum": ["Backlog", "Todo", "In Progress", "Done"], "description": "Move the task to this column." },
                "title": { "type": "string", "description": "Update task title." },
                "summary": { "type": "string", "description": "Update task details." },
                "assigned_agent": { "type": "string", "description": "Change task assignment." },
                "priority": { "type": "string", "enum": ["Low", "Medium", "High"], "description": "Change priority level." }
            },
            "required": ["id"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let id = args["id"].as_str().unwrap_or("").trim().to_string();
        if id.is_empty() {
            return Err("`id` is required".into());
        }

        let mut state = load_planner();
        let mut found = false;

        for t in &mut state.tasks {
            if t.id == id {
                found = true;
                if let Some(status_str) = args["status"].as_str() {
                    t.status = match status_str {
                        "Backlog" => TaskStatus::Backlog,
                        "In Progress" => TaskStatus::InProgress,
                        "Done" => TaskStatus::Done,
                        _ => TaskStatus::Todo,
                    };
                }
                if let Some(title) = args["title"].as_str() {
                    t.title = title.trim().to_string();
                }
                if let Some(summary) = args["summary"].as_str() {
                    t.summary = summary.trim().to_string();
                }
                if let Some(assigned) = args["assigned_agent"].as_str() {
                    t.assigned_agent = assigned.trim().to_string();
                }
                if let Some(priority) = args["priority"].as_str() {
                    t.priority = priority.to_string();
                }
                break;
            }
        }

        if !found {
            return Err(format!("No planner task found with ID '{}'.", id));
        }

        save_planner(&state)?;
        Ok(format!("Successfully updated task '{}'.", id))
    }
}

// ── DELETE PLANNER TASK TOOL ───────────────────────────────────────────────

pub struct DeletePlannerTask;

#[async_trait]
impl Tool for DeletePlannerTask {
    fn name(&self) -> &'static str {
        "delete_planner_task"
    }
    fn description(&self) -> &'static str {
        "Remove a task card from the Kanban planner entirely."
    }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "The unique ID of the task card to delete." }
            },
            "required": ["id"]
        })
    }
    fn is_sensitive(&self) -> bool {
        true
    }
    async fn run(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let id = args["id"].as_str().unwrap_or("").trim().to_string();
        if id.is_empty() {
            return Err("`id` is required".into());
        }

        let mut state = load_planner();
        let len_before = state.tasks.len();
        state.tasks.retain(|t| t.id != id);

        if state.tasks.len() == len_before {
            return Err(format!("No task found with ID '{}'.", id));
        }

        save_planner(&state)?;
        Ok(format!("Successfully deleted planner task '{}'.", id))
    }
}
