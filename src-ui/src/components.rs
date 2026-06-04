//! Shared UI primitives (the design-system library) + helper macros that bind a
//! form field to a `ServerConfig` field on the [`crate::state::AppCtx`].

use leptos::prelude::*;

#[component]
pub fn Spinner() -> impl IntoView {
    view! { <span class="spinner" aria-label="loading"></span> }
}

/// Page header: a display title + optional description, per DESIGN.md.
#[component]
pub fn PageHeader(
    title: &'static str,
    #[prop(optional, into)] desc: String,
) -> impl IntoView {
    let desc_view = (!desc.is_empty()).then(|| view! { <p class="page-desc">{desc}</p> });
    view! {
        <header class="page-header">
            <h1 class="page-title">{title}</h1>
            {desc_view}
        </header>
    }
}

/// A content card (DESIGN.md surface-card / feature-card).
#[component]
pub fn Card(#[prop(optional, into)] title: String, children: Children) -> impl IntoView {
    let title_view = (!title.is_empty()).then(|| view! { <div class="card-title">{title}</div> });
    view! {
        <section class="card">
            {title_view}
            {children()}
        </section>
    }
}

/// Text (or numeric) input field with a label + hint. Live-updates on input;
/// commits (persists) on change/blur.
#[component]
pub fn TextField(
    label: &'static str,
    value: Signal<String>,
    on_input: Callback<String>,
    on_commit: Callback<()>,
    #[prop(optional, into)] hint: String,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional)] numeric: bool,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let hint_view = (!hint.is_empty()).then(|| view! { <div class="field-hint">{hint}</div> });
    let input_id = if id.is_empty() { None } else { Some(id) };
    view! {
        <div class="field">
            <label class="field-label">{label}</label>
            <input
                class="input"
                id=input_id
                type=if numeric { "number" } else { "text" }
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |e| on_input.run(event_target_value(&e))
                on:change=move |_| on_commit.run(())
            />
            {hint_view}
        </div>
    }
}

/// Checkbox toggle field.
#[component]
pub fn ToggleField(
    label: &'static str,
    value: Signal<bool>,
    on_toggle: Callback<bool>,
    #[prop(optional, into)] hint: String,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let hint_view = (!hint.is_empty()).then(|| view! { <span class="field-hint">{hint}</span> });
    let input_id = if id.is_empty() { None } else { Some(id) };
    view! {
        <label class="toggle-field">
            <input
                type="checkbox"
                id=input_id
                prop:checked=move || value.get()
                on:change=move |e| on_toggle.run(event_target_checked(&e))
            />
            <span class="toggle-text">{label} {hint_view}</span>
        </label>
    }
}

/// Dropdown select field. `options` is a list of (value, label).
#[component]
pub fn SelectField(
    label: &'static str,
    value: Signal<String>,
    options: Vec<(String, String)>,
    on_select: Callback<String>,
    #[prop(optional, into)] id: String,
) -> impl IntoView {
    let select_id = if id.is_empty() { None } else { Some(id) };
    view! {
        <div class="field">
            <label class="field-label">{label}</label>
            <select
                class="input"
                id=select_id
                prop:value=move || value.get()
                on:change=move |e| on_select.run(event_target_value(&e))
            >
                {options
                    .into_iter()
                    .map(|(v, l)| view! { <option value=v>{l}</option> })
                    .collect_view()}
            </select>
        </div>
    }
}

// ── Field-binding macros ─────────────────────────────────────────────────────
// These collapse the boilerplate of wiring a form field to one `ServerConfig`
// field: read via a derived signal, write the working copy on input, persist on
// commit. `$ctx` must be a `Copy` AppCtx.

#[macro_export]
macro_rules! field_text {
    ($ctx:expr, $field:ident, $label:expr, $hint:expr) => {{
        let c = $ctx;
        let id_val = concat!("form-", stringify!($field));
        ::leptos::prelude::view! {
            <$crate::components::TextField
                label=$label hint=$hint id=id_val
                value=::leptos::prelude::Signal::derive(move || c.config.get().$field.clone())
                on_input=::leptos::prelude::Callback::new(move |v: ::std::string::String| {
                    c.config.update(|cc| cc.$field = v)
                })
                on_commit=::leptos::prelude::Callback::new(move |_| c.save())
            />
        }
    }};
}

#[macro_export]
macro_rules! field_num {
    ($ctx:expr, $field:ident, $ty:ty, $label:expr, $hint:expr) => {{
        let c = $ctx;
        let id_val = concat!("form-", stringify!($field));
        ::leptos::prelude::view! {
            <$crate::components::TextField
                label=$label hint=$hint numeric=true id=id_val
                value=::leptos::prelude::Signal::derive(move || c.config.get().$field.to_string())
                on_input=::leptos::prelude::Callback::new(move |v: ::std::string::String| {
                    if let Ok(p) = v.parse::<$ty>() { c.config.update(|cc| cc.$field = p) }
                })
                on_commit=::leptos::prelude::Callback::new(move |_| c.save())
            />
        }
    }};
}

#[macro_export]
macro_rules! field_bool {
    ($ctx:expr, $field:ident, $label:expr, $hint:expr) => {{
        let c = $ctx;
        let id_val = concat!("form-", stringify!($field));
        ::leptos::prelude::view! {
            <$crate::components::ToggleField
                label=$label hint=$hint id=id_val
                value=::leptos::prelude::Signal::derive(move || c.config.get().$field)
                on_toggle=::leptos::prelude::Callback::new(move |v: bool| {
                    c.config.update(|cc| cc.$field = v);
                    c.save();
                })
            />
        }
    }};
}
