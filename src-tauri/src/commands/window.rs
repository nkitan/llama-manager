//! Custom window-chrome controls. We run the window with native decorations
//! disabled and draw our own titlebar (so it's themeable), driving these from the
//! UI. App-defined commands need no capability entry.

use tauri::Window;

#[tauri::command]
pub fn win_minimize(window: Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn win_toggle_maximize(window: Window) {
    if matches!(window.is_maximized(), Ok(true)) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
pub fn win_close(window: Window) {
    let _ = window.close();
}
