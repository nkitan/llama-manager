//! Agent memory (L1 context). File-backed "lessons" the agent accumulates and
//! re-reads on later tasks. Stored as a JSON array under the config dir.

use std::path::PathBuf;

pub struct MemoryManager {
    path: PathBuf,
}

impl MemoryManager {
    pub fn new(config_dir: &PathBuf) -> Self {
        Self {
            path: config_dir.join("agent_memory.json"),
        }
    }

    fn load(&self) -> Vec<String> {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Render accumulated lessons, Obsidian memories, Notes, and Todos as a system-prompt fragment.
    pub fn context(&self) -> String {
        let mut s = String::new();

        // 1. Lessons
        let lessons = self.load();
        if !lessons.is_empty() {
            s.push_str("\nLessons learned from previous tasks:\n");
            for l in lessons.iter().take(20) {
                s.push_str(&format!("- {l}\n"));
            }
        }

        // 2. Obsidian Memories
        if let Ok(mems) = crate::commands::remaining::obsidian_memories_get() {
            if !mems.is_empty() {
                s.push_str("\nObsidian Knowledge Graph Memories:\n");
                for m in mems.iter().take(15) {
                    s.push_str(&format!("- Memory ID: {} (Title: \"{}\", Scope: {})\n", m.id, m.title, m.scope));
                    if !m.tags.is_empty() {
                        s.push_str(&format!("  Tags: {}\n", m.tags.join(", ")));
                    }
                    if !m.links.is_empty() {
                        s.push_str(&format!("  Links: {}\n", m.links.join(", ")));
                    }
                    let indented = m.content.replace('\n', "\n  ");
                    s.push_str(&format!("  Content:\n  \"\"\"\n  {}\n  \"\"\"\n", indented));
                }
            }
        }

        // 3. User Notes
        let global_notes = crate::commands::store::notes_store_get("global".to_string());
        let project_notes = crate::commands::store::notes_store_get("project".to_string());
        let mut has_notes = false;
        
        let mut notes_str = String::from("\nUser Notes (Global & Project):\n");
        for note in &global_notes.notes {
            if !note.content.trim().is_empty() {
                has_notes = true;
                let indented = note.content.replace('\n', "\n  ");
                notes_str.push_str(&format!("- Note Name: \"{}\" (Scope: global)\n  Content:\n  \"\"\"\n  {}\n  \"\"\"\n", note.name, indented));
            }
        }
        for note in &project_notes.notes {
            if !note.content.trim().is_empty() {
                has_notes = true;
                let indented = note.content.replace('\n', "\n  ");
                notes_str.push_str(&format!("- Note Name: \"{}\" (Scope: project)\n  Content:\n  \"\"\"\n  {}\n  \"\"\"\n", note.name, indented));
            }
        }
        if has_notes {
            s.push_str(&notes_str);
        }

        // 4. Todos
        let todos = crate::commands::store::todos_get();
        if !todos.is_empty() {
            s.push_str("\nUser Todos Checklist:\n");
            for t in todos {
                let check = if t.done { "[x]" } else { "[ ]" };
                s.push_str(&format!("- {} {} (id: {})\n", check, t.text, t.id));
            }
        }

        // 5. Scheduled Calendar Events
        let calendar = crate::commands::remaining::calendar_load();
        if !calendar.events.is_empty() {
            s.push_str("\nScheduled Calendar Events:\n");
            for ev in calendar.events {
                s.push_str(&format!(
                    "- [{:?}] Title: '{}', Time: {}, Trigger Prompt: '{}', Note: '{}', Result: '{}' (id: {})\n",
                    ev.status,
                    ev.title,
                    ev.time,
                    ev.prompt,
                    ev.note.as_deref().unwrap_or(""),
                    ev.result.as_deref().unwrap_or(""),
                    ev.id
                ));
            }
        }

        // 6. Task Planner Kanban Board
        let planner = crate::commands::remaining::planner_load();
        if !planner.tasks.is_empty() {
            s.push_str("\nTask Planner Kanban Board:\n");
            for t in planner.tasks {
                s.push_str(&format!(
                    "- [{:?}] Title: '{}', Summary: '{}', Assignee: {}, Priority: {}, Created At: {} (id: {})\n",
                    t.status,
                    t.title,
                    t.summary,
                    t.assigned_agent,
                    t.priority,
                    t.created_at,
                    t.id
                ));
            }
        }

        s
    }

    /// Append a lesson (best-effort; logs on failure rather than erroring the run).
    pub fn add_lesson(&self, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let mut lessons = self.load();
        lessons.push(text.to_string());
        if let Err(e) = std::fs::write(
            &self.path,
            serde_json::to_string_pretty(&lessons).unwrap_or_default(),
        ) {
            tracing::warn!(%e, "failed to persist agent lesson");
        }
    }
}
