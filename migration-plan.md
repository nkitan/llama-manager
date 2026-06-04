# Migration Plan: Dioxus to Tauri v2 + Leptos

## 1. Target Architecture: Tauri v2 + Leptos

Based on the requirement for a mature, bug-free cross-platform desktop framework using Rust (especially on Linux/Wayland), the recommended stack is **Tauri v2 + Leptos**.

### Why this stack?
* **Tauri v2:** Tauri is the industry standard for Rust desktop applications. It leverages the OS's native WebViews (WebKitGTK on Linux) and handles Wayland compositing flawlessly. It abstracts away the windowing quirks that `dioxus-desktop` (which relies directly on `tao`/`winit`) struggles with.
* **Leptos:** Since you want a "full-stack Rust" experience, Leptos is currently the most robust and mature web framework in Rust. It features fine-grained reactivity (no virtual DOM overhead) and translates beautifully to WebAssembly.

---

## 2. Current State Analysis

The current `llama-manager` project is a monolith built with Dioxus:
* **Massive `main.rs`:** `src/main.rs` is nearly 11,000 lines long, combining UI components, state management, and direct system access (spawning processes, reading files).
* **Direct OS Access in UI:** The Dioxus app directly accesses the file system and spawns subprocesses natively. In a Webview-based framework like Tauri, the UI layer runs in WebAssembly and cannot directly access the host OS for security and architectural reasons.

---

## 3. Step-by-Step Migration Plan

### Step 1: Architectural Decoupling (Prep Phase)
Before jumping into the new framework, we must untangle the UI logic from the backend operations.
* **Identify System Operations:** Extract all logic that interacts with the OS (`std::process::Command`, `std::fs`, `tokio::process`, file watchers, etc.) into isolated pure Rust modules.
* **Identify Global State:** The current app likely relies on `use_shared_state` or global `Mutex`es. Map out the shared state required by different tabs (Chat, Planner, Monitor).

### Step 2: Initialize Workspace
Create a modern Cargo workspace to separate the frontend from the backend.
```text
llama-manager/
├── Cargo.toml          (Workspace root)
├── src-tauri/          (Backend: Rust + Tauri Core)
│   ├── Cargo.toml
│   └── src/
└── src-ui/             (Frontend: Rust + Leptos + WASM)
    ├── Cargo.toml
    ├── index.html
    └── src/
```

### Step 3: Implement the Tauri Backend (IPC Layer)
Migrate the isolated system logic into `src-tauri` and expose them as Tauri commands.
* Convert backend functions to `#[tauri::command]`.
* *Example:* Instead of spawning the `llama-server` directly inside a UI click handler, create a `start_server()` command in Tauri that the frontend will invoke.
* Implement Tauri state management (`tauri::State`) to manage persistent background tasks (like tracking the running server instance).

### Step 4: UI Translation (Dioxus RSX -> Leptos View)
This is the most labor-intensive step but yields massive performance and maintainability gains.
* **Modularization:** Break down the monolithic `main.rs` into logical components (e.g., `src-ui/src/tabs/chat.rs`, `src-ui/src/tabs/planner.rs`).
* **State Management Migration:**
  * Dioxus: `let mut val = use_signal(|| 0);`
  * Leptos: `let (val, set_val) = create_signal(0);`
* **UI Macros:** Translate `rsx! {}` to `view! {}`. The syntax is very similar, but Leptos enforces closures for dynamic values (e.g., `class=move || ...`).
* **CSS:** The existing global CSS block can be moved directly into a `style.css` file imported by `index.html`.

### Step 5: Bridging Frontend and Backend (IPC)
Update the Leptos UI to communicate with the Tauri backend.
* Use the `tauri-sys` crate or `wasm-bindgen` to invoke Tauri commands from Leptos.
* Example:
  ```rust
  use serde::{Deserialize, Serialize};
  use wasm_bindgen::prelude::*;

  #[wasm_bindgen]
  extern "C" {
      #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
      async fn invoke(cmd: &str, args: JsValue) -> JsValue;
  }
  ```

### Step 6: Wayland Testing & Build Configuration
* Since Tauri handles the window creation via WebKitGTK, the Wayland ghosting and transparency bugs should immediately be resolved.
* Ensure `tauri.conf.json` enables the necessary APIs (e.g., `fs`, `process`).
* Configure the build pipeline to compile Leptos to WASM (`trunk` or `cargo-leptos`) and bundle it via the Tauri CLI.

## 4. Effort Estimation and Risks

* **Effort:** High. Migrating 11k lines of UI code is a significant refactor. Modularizing the `main.rs` file during the process will be crucial.
* **Risk:** WebAssembly compilation restricts native crate usage in the frontend. Any dependency currently used in Dioxus that relies on native C-bindings (or isn't `wasm32-unknown-unknown` compatible) must be moved to the `src-tauri` backend layer.
* **Benefit:** A highly stable, performant, and Wayland-native desktop application with a clean separation of concerns.
