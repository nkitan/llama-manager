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
        items.update(|l| l.push(TodoItem { id, text: t, done: false }));
        new_text.set(String::new());
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
                <div class="composer-inline">
                    <input
                        class="input"
                        placeholder="Add a todo…"
                        prop:value=move || new_text.get()
                        on:input=move |e| new_text.set(event_target_value(&e))
                        on:keydown=move |e: KeyboardEvent| if e.key() == "Enter" { add() }
                    />
                    <button class="btn primary" on:click=move |_| add()>"Add"</button>
                </div>
                <ul class="todo-list">
                    {move || {
                        items
                            .get()
                            .into_iter()
                            .map(|it| {
                                let (id1, id2) = (it.id.clone(), it.id.clone());
                                view! {
                                    <li class="todo-item" class:done=it.done>
                                        <label class="todo-label">
                                            <input
                                                type="checkbox"
                                                prop:checked=it.done
                                                on:change=move |_| toggle(id1.clone())
                                            />
                                            <span>{it.text.clone()}</span>
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

            <div style="display: flex; gap: var(--s-lg); align-items: flex-start; width: 100%;">
                
                // Notes Sidebar
                <div style="width: 240px; flex-shrink: 0;">
                    <Card title="">
                        <div style="padding: 12px; display: flex; flex-direction: column; gap: 12px;">
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

                            <div style="display: flex; flex-direction: column; gap: 6px; overflow-y: auto; max-height: 400px; min-height: 100px;">
                                {move || {
                                    let current_idx = active_note_idx.get();
                                    let notes_list = store.get().notes;
                                    
                                    notes_list.into_iter().enumerate().map(|(idx, note)| {
                                        let is_active = idx == current_idx;
                                        let show_ren = show_rename.get() == Some(idx);
                                        let n_name = note.name.clone();
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
                                                        <span style="font-size: 13px; text-overflow: ellipsis; overflow: hidden; white-space: nowrap; max-width: 140px;">
                                                            {note.name.clone()}
                                                        </span>
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
                    </Card>
                </div>

                // Editor Pane
                <div style="flex: 1; min-height: 480px;">
                    <Card title="">
                        <div style="padding: 16px; display: flex; flex-direction: column; gap: var(--s-md); min-height: 450px;">
                            {move || {
                                let st = store.get();
                                let idx = active_note_idx.get();
                                if let Some(note) = st.notes.get(idx) {
                                    let note_content = note.content.clone();
                                    view! {
                                        <div style="display: flex; flex-direction: column; gap: var(--s-md); height: 100%; flex: 1;">
                                            <div style="font-size: 15px; font-weight: 600; color: var(--color-text-header);">
                                                {format!("Editing: {}", note.name)}
                                            </div>
                                            <textarea
                                                class="notes-area"
                                                style="flex: 1; min-height: 350px; font-family: monospace; font-size: 13.5px; line-height: 1.5; resize: vertical; width: 100%; box-sizing: border-box;"
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

                            <div class="row-actions" style="margin-top: 12px; display: flex; align-items: center;">
                                <button class="btn primary" on:click=move |_| save_current()>"Save"</button>
                                <span class="field-hint" style="margin-left: 12px; font-size: 12px; color: var(--color-text-body); opacity: 0.8;">
                                    {move || save_status.get()}
                                </span>
                            </div>
                        </div>
                    </Card>
                </div>

            </div>
        </div>
    }
}
