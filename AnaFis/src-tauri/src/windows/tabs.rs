// src-tauri/src/tabs.rs

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Emitter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub content_type: String,
    pub state: serde_json::Value,
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub async fn send_tab_to_main(
    app_handle: AppHandle,
    tab_info: TabInfo,
    window_id: String,
) -> Result<(), String> {
    // Find the main window
    let main_window = app_handle.get_webview_window("main")
        .ok_or("Main window not found")?;

    // Emit the event to the main window to add the tab
    main_window.emit("tab-from-detached", &tab_info)
        .map_err(|e| format!("Failed to send tab to main: {}", e))?;

    // Close the detached window
    if let Some(detached_window) = app_handle.get_webview_window(&window_id) {
        let _ = detached_window.close();
    }

    Ok(())
}
