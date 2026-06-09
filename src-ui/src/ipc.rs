//! Typed bridge to the Tauri backend. The ONLY place the frontend touches
//! `window.__TAURI__` (see [[migration-conventions]]); every component calls the
//! typed wrappers below.
//!
//! Request/response → [`invoke`]. Backend→frontend streams → [`listen`], which
//! decodes the event payload into our `shared::ipc` enums.

use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    // window.__TAURI__.core.invoke(cmd, args) -> Promise
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    fn tauri_invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    // window.__TAURI__.event.listen(event, handler) -> Promise<UnlistenFn>
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = listen)]
    fn tauri_listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> js_sys::Promise;
}

/// Invoke a backend command. `args` is serialized to a JS object whose keys must
/// match the command's parameter names (camelCase ↔ snake_case is handled by
/// Tauri). Use [`no_args`] for commands without parameters.
pub async fn invoke<A: Serialize, R: DeserializeOwned>(cmd: &str, args: &A) -> Result<R, String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let value = JsFuture::from(tauri_invoke(cmd, args))
        .await
        .map_err(|e| format!("invoke `{cmd}` failed: {e:?}"))?;
    serde_wasm_bindgen::from_value(value).map_err(|e| e.to_string())
}

/// Empty argument object for parameterless commands.
pub fn no_args() -> serde_json::Value {
    serde_json::json!({})
}

/// Subscribe to a backend event channel, decoding each payload into `T`.
///
/// The registration is async (Tauri returns a Promise); set listeners up at
/// startup before triggering work. The handler closure is intentionally leaked
/// (`forget`) so it lives for the app's lifetime — listeners here are global and
/// never torn down.
pub fn listen<T, F>(event: &'static str, mut on_event: F)
where
    T: DeserializeOwned + 'static,
    F: FnMut(T) + 'static,
{
    let closure = Closure::wrap(Box::new(move |raw: JsValue| {
        // Tauri delivers `{ event, id, payload }`; we want `payload`.
        let payload = js_sys::Reflect::get(&raw, &JsValue::from_str("payload"))
            .unwrap_or(JsValue::NULL);
        match serde_wasm_bindgen::from_value::<T>(payload) {
            Ok(value) => on_event(value),
            Err(e) => web_sys::console::error_1(
                &format!("failed to decode `{event}` payload: {e}").into(),
            ),
        }
    }) as Box<dyn FnMut(JsValue)>);

    let _ = tauri_listen(event, &closure);
    closure.forget();
}
