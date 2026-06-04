//! Leptos CSR entry point. Compiled to wasm and mounted into the Tauri WebView.

mod api;
mod app;
mod components;
mod ipc;
mod state;
mod tabs;
mod theme;

fn main() {
    // Readable panics + structured logs in the browser console (frontend half of
    // the logging standard in [[migration-conventions]]).
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    leptos::mount::mount_to_body(app::App);
}
