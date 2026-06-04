//! Todos + Quick Notes tabs. Persisted via backend commands (`todos_*`,
//! `notes_*`); the same files the agent's productivity tools read/write.

use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use shared::ipc::{TodoItem, Note, NotesStore};
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::{Card, PageHeader};

#[component]
pub fn TodosTab() -> impl IntoView {
    let items = RwSignal::new(Vec::<TodoItem>::new());
    let new_text = RwSignal::new(String::new());
    let show_info = RwSignal::new(false);
    let filter_priority = RwSignal::new("all".to_string());
    let show_done_items = RwSignal::new(true);
    let new_priority = RwSignal::new(String::new());
    let new_due_date = RwSignal::new(String::new());

    spawn_local(async move {
        if let Ok(v) = api::todos_get().await {
            items.set(v);
        }
    });

    let persist = move || {
        let v = items.get_untracked();
        spawn_local(async move {
            let _ = api::todos_set(v).await;
        });
    };
    let add = move || {
        let t = new_text.get_untracked().trim().to_string();
        if t.is_empty() {
            return;
        }
        let id = format!("t{}", js_sys::Date::now() as u64);
        items.update(|l| l.push(TodoItem {
            id,
            text: t,
            done: false,
            priority: new_priority.get_untracked(),
            due_date: { let d = new_due_date.get_untracked(); if d.is_empty() { None } else { Some(d) } },
            tags: vec![],
        }));
        new_text.set(String::new());
        new_priority.set(String::new());
        new_due_date.set(String::new());
        persist();
    };
    let toggle = move |id: String| {
        items.update(|l| {
            for it in l.iter_mut() {
                if it.id == id {
                    it.done = !it.done;
                }
            }
        });
        persist();
    };
    let remove = move |id: String| {
        items.update(|l| l.retain(|i| i.id != id));
        persist();
    };

    view! {
        <div class="page">
            <PageHeader title="Todos" desc="A simple checklist — also visible to agents."/>

            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; margin-top:-8px;">
                <span style="font-size:12px; color:var(--muted);">"AI Agent checklist integration"</span>
                <button class="btn-info-toggle" on:click=move |_| show_info.update(|v| *v = !*v)>"i"</button>
            </div>

            {move || show_info.get().then(|| view! {
                <div class="reference-info-box">
                    <h4>"ℹ️ Referencing Todos in AI Agent"</h4>
                    "The AI agent can access your checklist using the "<code>"get_todos"</code>", "<code>"add_todo"</code>", and "<code>"complete_todo"</code>" tools. You can instruct the agent in Chat or Monitor tabs using prompts like:"
                    <ul>
                        <li>"\"Check my checklist and tell me what is left to do.\""</li>
                        <li>"\"Add a new todo item: Deploy update v2.0\""</li>
                        <li>"\"Mark todo item t171228 as completed.\""</li>
                        <li>"\"Delete my todo item with id t171228.\""</li>
                    </ul>
                </div>
            })}

            <Card title="">
                <div class="composer-inline" style="flex-wrap: wrap; gap: 6px;">
                    <input
                        class="input"
                        style="flex: 2; min-width: 160px;"
                        placeholder="Add a todo…"
                        prop:value=move || new_text.get()
                        on:input=move |e| new_text.set(event_target_value(&e))
                        on:keydown=move |e: KeyboardEvent| if e.key() == "Enter" { add() }
                    />
                    <select
                        class="input"
                        style="height: 34px; flex: 0 0 110px; font-size: 12px;"
                        prop:value=move || new_priority.get()
                        on:change=move |e| new_priority.set(event_target_value(&e))
                    >
                        <option value="">"— Priority"</option>
                        <option value="High">"🔴 High"</option>
                        <option value="Medium">"🟡 Medium"</option>
                        <option value="Low">"🔵 Low"</option>
                    </select>
                    <input
                        class="input"
                        type="date"
                        style="flex: 0 0 140px; height: 34px;"
                        prop:value=move || new_due_date.get()
                        on:input=move |e| new_due_date.set(event_target_value(&e))
                    />
                    <button class="btn primary" on:click=move |_| add()>"Add"</button>
                </div>

                <div style="display: flex; gap: 6px; align-items: center; margin-top: 10px; margin-bottom: 8px; flex-wrap: wrap;">
                    <span style="font-size: 11px; color: var(--muted);">"Filter:"</span>
                    {["all", "High", "Medium", "Low"].into_iter().map(|p| {
                        let p_str = p.to_string();
                        let p_str2 = p_str.clone();
                        let p_str3 = p_str.clone();
                        let label = match p { "all" => "All", other => other };
                        view! {
                            <button
                                class=move || if filter_priority.get() == p_str2 { "btn primary sm" } else { "btn secondary sm" }
                                style="font-size: 11px; padding: 2px 8px; height: 24px;"
                                on:click=move |_| filter_priority.set(p_str3.clone())
                            >{label}</button>
                        }
                    }).collect_view()}
                    <button
                        class=move || if show_done_items.get() { "btn primary sm" } else { "btn secondary sm" }
                        style="font-size: 11px; padding: 2px 8px; height: 24px; margin-left: 8px;"
                        on:click=move |_| show_done_items.update(|v| *v = !*v)
                    >
                        {move || if show_done_items.get() { "Hide Done" } else { "Show Done" }}
                    </button>
                </div>

                <ul class="todo-list">
                    {move || {
                        let fp = filter_priority.get();
                        let show_done = show_done_items.get();
                        items
                            .get()
                            .into_iter()
                            .filter(|it| {
                                let priority_ok = fp == "all" || it.priority == fp;
                                let done_ok = show_done || !it.done;
                                priority_ok && done_ok
                            })
                            .map(|it| {
                                let (id1, id2) = (it.id.clone(), it.id.clone());
                                let priority_badge = if !it.priority.is_empty() {
                                    let color = match it.priority.as_str() {
                                        "High" => "#ef4444",
                                        "Medium" => "#f59e0b",
                                        "Low" => "#3b82f6",
                                        _ => "var(--muted)",
                                    };
                                    let icon = match it.priority.as_str() {
                                        "High" => "🔴",
                                        "Medium" => "🟡",
                                        "Low" => "🔵",
                                        _ => "",
                                    };
                                    let p = it.priority.clone();
                                    Some(view! {
                                        <span style=format!("font-size: 10px; font-weight: 600; color: {}; background: color-mix(in srgb, {} 15%, transparent); border-radius: 3px; padding: 1px 5px;", color, color)>
                                            {format!("{} {}", icon, p)}
                                        </span>
                                    })
                                } else {
                                    None
                                };
                                let due_badge = it.due_date.as_ref().map(|d| {
                                    let d = d.clone();
                                    view! {
                                        <span style="font-size: 10px; color: var(--muted); font-family: var(--font-mono);">
                                            {format!("📅 {}", d)}
                                        </span>
                                    }
                                });
                                view! {
                                    <li class="todo-item" class:done=it.done>
                                        <label class="todo-label">
                                            <input
                                                type="checkbox"
                                                prop:checked=it.done
                                                on:change=move |_| toggle(id1.clone())
                                            />
                                            <div style="display: flex; flex-direction: column; gap: 2px;">
                                                <span>{it.text.clone()}</span>
                                                <div style="display: flex; gap: 4px; align-items: center; flex-wrap: wrap;">
                                                    {priority_badge}
                                                    {due_badge}
                                                </div>
                                            </div>
                                        </label>
                                        <button
                                            class="btn ghost sm"
                                            on:click=move |_| remove(id2.clone())
                                        >
                                            "✕"
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
            </Card>
        </div>
    }
}

#[component]
pub fn NotesTab() -> impl IntoView {
    let scope = RwSignal::new("project".to_string());
    let store = RwSignal::new(NotesStore::default());
    let active_note_idx = RwSignal::new(0usize);
    let new_note_name = RwSignal::new(String::new());
    let show_rename = RwSignal::new(None::<usize>);
    let rename_value = RwSignal::new(String::new());
    let save_status = RwSignal::new("Saved".to_string());
    let show_info = RwSignal::new(false);
    let note_search = RwSignal::new(String::new());
    let new_tag_input = RwSignal::new(String::new());

    // Effect to reload when scope changes
    Effect::new(move || {
        let sc = scope.get();
        spawn_local(async move {
            if let Ok(st) = api::notes_store_get(sc).await {
                store.set(st);
                active_note_idx.set(0);
                save_status.set("Saved".to_string());
            }
        });
    });

    // Periodic refresh to pick up AI/agent changes
    let scope_ref = scope.clone();
    let store_ref = store.clone();
    let save_status_ref = save_status.clone();
    spawn_local(async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            let sc = scope_ref.get_untracked();
            let current_status = save_status_ref.get_untracked();
            if current_status == "Saved" {
                if let Ok(st) = api::notes_store_get(sc).await {
                    let untracked_store = store_ref.get_untracked();
                    if untracked_store != st {
                        store_ref.set(st);
                    }
                }
            }
        }
    });

    let save_current = move || {
        let sc = scope.get_untracked();
        let st = store.get_untracked();
        spawn_local(async move {
            match api::notes_store_set(sc, st).await {
                Ok(_) => {
                    let now = js_sys::Date::new_0().to_locale_time_string("en-US");
                    let clean_time = now.as_string().unwrap_or_else(|| "recent".to_string());
                    save_status.set(format!("Saved at {}", clean_time));
                }
                Err(e) => {
                    save_status.set(format!("Error: {}", e));
                }
            }
        });
    };

    let add_note = move || {
        let name = new_note_name.get_untracked().trim().to_string();
        let name = if name.is_empty() {
            format!("Note {}", store.get_untracked().notes.len() + 1)
        } else {
            name
        };
        store.update(|st| {
            st.notes.push(Note {
                name,
                content: String::new(),
                tags: vec![],
                pinned: false,
            });
        });
        let new_idx = store.get_untracked().notes.len() - 1;
        active_note_idx.set(new_idx);
        new_note_name.set(String::new());
        save_current();
    };

    let delete_note = move |idx: usize| {
        store.update(|st| {
            if st.notes.len() > 1 && idx < st.notes.len() {
                st.notes.remove(idx);
            } else if st.notes.len() == 1 {
                st.notes[0].content = String::new();
                st.notes[0].name = "Default".to_string();
            }
        });
        let current_idx = active_note_idx.get_untracked();
        if current_idx >= store.get_untracked().notes.len() {
            active_note_idx.set((store.get_untracked().notes.len() as isize - 1).max(0) as usize);
        }
        save_current();
    };

    let rename_note_save = move |idx: usize| {
        let name = rename_value.get_untracked().trim().to_string();
        if !name.is_empty() {
            store.update(|st| {
                if idx < st.notes.len() {
                    st.notes[idx].name = name;
                }
            });
            save_current();
        }
        show_rename.set(None);
    };

    view! {
        <div class="page">
            <PageHeader title="Quick Notes" desc="Multiple named notes per scope. The AI agent can read and edit these files."/>

            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; margin-top:-8px;">
                <span style="font-size:12px; color:var(--muted);">"AI Agent notes integration"</span>
                <button class="btn-info-toggle" on:click=move |_| show_info.update(|v| *v = !*v)>"i"</button>
            </div>

            {move || show_info.get().then(|| view! {
                <div class="reference-info-box">
                    <h4>"ℹ️ Referencing Notes in AI Agent"</h4>
                    "The AI agent reads and writes your quick notes via "<code>"read_notes"</code>" and "<code>"write_note"</code>" tools. You can instruct the agent to query or modify these notes in Chat or Monitor tabs:"
                    <ul>
                        <li>"\"Read the note named 'Setup Instructions' in project scope and summarize the steps.\""</li>
                        <li>"\"Write a note named 'Daily Report' in global scope with today's achievements.\""</li>
                        <li>"\"Append 'Deployment confirmed' to the 'Release Checklist' note in global scope.\""</li>
                        <li>"\"Understand the note 'Architecture' and list any architectural design issues.\""</li>
                    </ul>
                </div>
            })}
            
            <div style="display: flex; gap: 8px; margin-bottom: var(--s-md);">
                <button
                    class=move || if scope.get() == "project" { "btn primary sm" } else { "btn secondary sm" }
                    on:click=move |_| scope.set("project".to_string())
                >
                    "📂 Project Notes"
                </button>
                <button
                    class=move || if scope.get() == "global" { "btn primary sm" } else { "btn secondary sm" }
                    on:click=move |_| scope.set("global".to_string())
                >
                    "🌐 Global Notes"
                </button>
            </div>

            <div style="display: flex; gap: var(--s-lg); align-items: stretch; width: 100%; min-height: 500px;">

                // Notes Sidebar
                <div style="width: 240px; flex-shrink: 0; display: flex; flex-direction: column;">
                    <section class="card" style="flex: 1; display: flex; flex-direction: column; padding: 0; overflow: hidden;">
                        <div style="padding: 12px; display: flex; flex-direction: column; gap: 12px; flex: 1; overflow: hidden;">
                            <div style="display: flex; justify-content: space-between; align-items: center;">
                                <span style="font-weight: 600; font-size: 14px; color: var(--color-text-header);">"Notes"</span>
                                <button
                                    class="btn ghost sm"
                                    style="padding: 2px 6px; font-size: 11px;"
                                    title="Add a new note"
                                    on:click=move |_| add_note()
                                >
                                    "➕"
                                </button>
                            </div>

                            <input
                                class="input"
                                style="font-size: 12px; padding: var(--s-xs) var(--s-sm); height: 30px; width: 100%; box-sizing: border-box;"
                                placeholder="Add note name..."
                                prop:value=move || new_note_name.get()
                                on:input=move |e| new_note_name.set(event_target_value(&e))
                                on:keydown=move |e: KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        add_note();
                                    }
                                }
                            />

                            <input
                                class="input"
                                style="font-size: 12px; padding: var(--s-xs) var(--s-sm); height: 28px; width: 100%; box-sizing: border-box;"
                                placeholder="Search notes..."
                                prop:value=move || note_search.get()
                                on:input=move |e| note_search.set(event_target_value(&e))
                            />

                            <div style="display: flex; flex-direction: column; gap: 6px; overflow-y: auto; flex: 1; min-height: 100px;">
                                {move || {
                                    let current_idx = active_note_idx.get();
                                    let search_q = note_search.get().to_lowercase();
                                    let notes_list = store.get().notes;

                                    notes_list.into_iter().enumerate().filter(|(_, note)| {
                                        search_q.is_empty()
                                            || note.name.to_lowercase().contains(&search_q)
                                            || note.content.to_lowercase().contains(&search_q)
                                            || note.tags.iter().any(|t| t.to_lowercase().contains(&search_q))
                                    }).map(|(idx, note)| {
                                        let is_active = idx == current_idx;
                                        let show_ren = show_rename.get() == Some(idx);
                                        let n_name = note.name.clone();
                                        let note_tags = note.tags.clone();
                                        view! {
                                            <div
                                                style=format!(
                                                    "display: flex; align-items: center; justify-content: space-between; padding: 8px 10px; border-radius: var(--r-md); cursor: pointer; transition: all 0.2s; {}",
                                                    if is_active { "background: var(--color-btn-primary-bg); color: var(--color-btn-primary-text); font-weight: 600;" } else { "background: rgba(255,255,255,0.03); color: var(--color-text-body);" }
                                                )
                                                on:click=move |_| active_note_idx.set(idx)
                                            >
                                                {if show_ren {
                                                    view! {
                                                        <input
                                                            class="input"
                                                            style="font-size: 11px; padding: 2px 4px; height: 22px; width: 120px;"
                                                            prop:value=move || rename_value.get()
                                                            on:input=move |e| rename_value.set(event_target_value(&e))
                                                            on:keydown=move |e: KeyboardEvent| {
                                                                if e.key() == "Enter" {
                                                                    rename_note_save(idx);
                                                                } else if e.key() == "Escape" {
                                                                    show_rename.set(None);
                                                                }
                                                            }
                                                            on:blur=move |_| rename_note_save(idx)
                                                        />
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div style="display: flex; flex-direction: column; gap: 2px; min-width: 0;">
                                                            <span style="font-size: 13px; text-overflow: ellipsis; overflow: hidden; white-space: nowrap; max-width: 140px;">
                                                                {note.name.clone()}
                                                            </span>
                                                            {if !note_tags.is_empty() {
                                                                view! {
                                                                    <div style="display: flex; gap: 3px; flex-wrap: wrap;">
                                                                        {note_tags.iter().map(|tag| {
                                                                            let t = tag.clone();
                                                                            view! {
                                                                                <span style="font-size: 9px; background: color-mix(in srgb, var(--accent) 20%, transparent); color: var(--accent); border-radius: 3px; padding: 0 4px; line-height: 1.6;">{t}</span>
                                                                            }
                                                                        }).collect_view()}
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                view! {}.into_any()
                                                            }}
                                                        </div>
                                                    }.into_any()
                                                }}

                                                <div style="display: flex; gap: 2px;">
                                                    {if !show_ren {
                                                        let n_name2 = n_name.clone();
                                                        view! {
                                                            <button
                                                                class="btn ghost sm"
                                                                style="padding: 1px 4px; font-size: 10px; color: inherit; opacity: 0.6;"
                                                                on:click=move |e| {
                                                                    e.stop_propagation();
                                                                    rename_value.set(n_name2.clone());
                                                                    show_rename.set(Some(idx));
                                                                }
                                                            >
                                                                "✏️"
                                                            </button>
                                                        }.into_any()
                                                    } else {
                                                        view! {}.into_any()
                                                    }}
                                                    <button
                                                        class="btn ghost sm"
                                                        style="padding: 1px 4px; font-size: 10px; color: #ef4444; opacity: 0.6;"
                                                        on:click=move |e| {
                                                            e.stop_propagation();
                                                            delete_note(idx);
                                                        }
                                                    >
                                                        "✕"
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()
                                }}
                            </div>
                        </div>
                    </section>
                </div>

                // Editor Pane
                <div style="flex: 1; display: flex; flex-direction: column;">
                    <section class="card" style="flex: 1; display: flex; flex-direction: column; padding: 0; overflow: hidden;">
                        <div style="padding: 16px; display: flex; flex-direction: column; gap: var(--s-md); flex: 1; min-height: 0;">
                            {move || {
                                let st = store.get();
                                let idx = active_note_idx.get();
                                if let Some(note) = st.notes.get(idx) {
                                    let note_content = note.content.clone();
                                    let note_name = note.name.clone();
                                    let note_tags_editor = note.tags.clone();
                                    view! {
                                        <div style="display: flex; flex-direction: column; gap: var(--s-sm); flex: 1; height: 100%;">
                                            <div style="font-size: 13.5px; font-weight: 600; color: var(--color-text-header);">
                                                {format!("✏ {}", note_name)}
                                            </div>
                                            // Tags editor
                                            <div style="display: flex; gap: 6px; align-items: center; flex-wrap: wrap; padding: 4px 0;">
                                                <span style="font-size: 11px; color: var(--muted);">"Tags:"</span>
                                                {note_tags_editor.iter().enumerate().map(|(ti, tag)| {
                                                    let t = tag.clone();
                                                    view! {
                                                        <span style="display: inline-flex; align-items: center; gap: 3px; font-size: 11px; background: color-mix(in srgb, var(--accent) 15%, transparent); color: var(--accent); border-radius: 4px; padding: 1px 6px;">
                                                            {t.clone()}
                                                            <button
                                                                style="border: none; background: none; cursor: pointer; font-size: 10px; color: var(--accent); padding: 0; line-height: 1;"
                                                                on:click=move |_| {
                                                                    store.update(|st| {
                                                                        if idx < st.notes.len() && ti < st.notes[idx].tags.len() {
                                                                            st.notes[idx].tags.remove(ti);
                                                                        }
                                                                    });
                                                                    save_status.set("Unsaved changes".to_string());
                                                                }
                                                            >"×"</button>
                                                        </span>
                                                    }
                                                }).collect_view()}
                                                <input
                                                    style="font-size: 11px; height: 22px; padding: 0 6px; border: var(--border-width) solid var(--hairline); border-radius: 4px; background: var(--canvas); color: var(--ink); min-width: 80px; max-width: 120px;"
                                                    placeholder="Add tag..."
                                                    prop:value=move || new_tag_input.get()
                                                    on:input=move |e| new_tag_input.set(event_target_value(&e))
                                                    on:keydown=move |e: KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            let tag = new_tag_input.get_untracked().trim().to_string();
                                                            if !tag.is_empty() {
                                                                store.update(|st| {
                                                                    if idx < st.notes.len() && !st.notes[idx].tags.contains(&tag) {
                                                                        st.notes[idx].tags.push(tag.clone());
                                                                    }
                                                                });
                                                                new_tag_input.set(String::new());
                                                                save_status.set("Unsaved changes".to_string());
                                                            }
                                                        }
                                                    }
                                                />
                                            </div>
                                            // Markdown toolbar
                                            <div style="display: flex; gap: 4px; flex-wrap: wrap; padding: 6px 8px; background: var(--surface-soft); border: var(--border-width) solid var(--hairline); border-radius: var(--r-sm); border-bottom: none; border-bottom-left-radius: 0; border-bottom-right-radius: 0;">
                                                {[("B", "**bold**"), ("I", "_italic_"), ("~~", "~~strike~~"), ("H1", "# "), ("H2", "## "), ("H3", "### "), ("`", "`code`"), ("```", "```\n\n```"), ("—", "---\n"), ("• ", "- item"), ("1.", "1. item"), ("[ ]", "- [ ] task"), ("🔗", "[text](url)"), ("\"", "> quote")].into_iter().map(|(label, snippet)| {
                                                    let sn = snippet.to_string();
                                                    view! {
                                                        <button
                                                            style="padding: 2px 7px; font-size: 11.5px; font-weight: 600; font-family: var(--font-mono); border: var(--border-width) solid var(--hairline); border-radius: var(--r-sm); background: var(--canvas); color: var(--body); cursor: pointer; line-height: 1.6;"
                                                            title=snippet
                                                            on:click=move |_| {
                                                                store.update(|st| {
                                                                    if idx < st.notes.len() {
                                                                        st.notes[idx].content.push_str(&sn);
                                                                    }
                                                                });
                                                                save_status.set("Unsaved changes".to_string());
                                                            }
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>
                                            <textarea
                                                class="notes-area"
                                                style="flex: 1; min-height: 300px; font-family: var(--font-mono); font-size: 13.5px; line-height: 1.6; resize: vertical; width: 100%; box-sizing: border-box; border-top-left-radius: 0; border-top-right-radius: 0;"
                                                prop:value=note_content
                                                on:input=move |e| {
                                                    let val = event_target_value(&e);
                                                    store.update(|st| {
                                                        if idx < st.notes.len() {
                                                            st.notes[idx].content = val;
                                                        }
                                                    });
                                                    save_status.set("Unsaved changes".to_string());
                                                }
                                            ></textarea>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="text-align: center; color: var(--color-muted); padding: 40px;">
                                            "Select or add a note."
                                        </div>
                                    }.into_any()
                                }
                            }}

                            <div class="row-actions" style="margin-top: 8px; display: flex; align-items: center;">
                                <button class="btn primary" on:click=move |_| save_current()>"Save"</button>
                                <span class="field-hint" style="margin-left: 12px; font-size: 12px; opacity: 0.8;">
                                    {move || save_status.get()}
                                </span>
                            </div>
                        </div>
                    </section>
                </div>

            </div>
        </div>
    }
}
