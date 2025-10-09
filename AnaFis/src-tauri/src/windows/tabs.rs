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
        .map_err(|e| format!("Failed to send tab to main: {e}"))?;

    // Close the detached window
    if let Some(detached_window) = app_handle.get_webview_window(&window_id) {
        let _ = detached_window.close();
    }

    Ok(())
}

#[tauri::command]
pub async fn create_tab_window(
    app_handle: AppHandle,
    tab_info: TabInfo,
    geometry: Option<WindowGeometry>,
) -> Result<(), String> {
    let window_label = format!("tab_{}", tab_info.id);
    let url = format!(
        "tab.html?tabId={}&tabTitle={}&tabType={}",
        urlencoding::encode(&tab_info.id),
        urlencoding::encode(&tab_info.title),
        urlencoding::encode(&tab_info.content_type)
    );

    let width = geometry.as_ref().map(|g| g.width).unwrap_or(800);
    let height = geometry.as_ref().map(|g| g.height).unwrap_or(600);

    let config = crate::windows::window_manager::WindowConfig {
        title: tab_info.title.clone(),
        url,
        width: width as f64,
        height: height as f64,
        resizable: true,
        decorations: false,
        transparent: false,
        always_on_top: true,
        skip_taskbar: false,
        parent: None,
        min_width: Some(600.0),
        min_height: Some(400.0),
    };

    crate::windows::window_manager::create_or_focus_window(&app_handle, &window_label, config)
        .map_err(|e| format!("Failed to create tab window: {e}"))?;

    // If geometry provided, set position
    if let Some(geom) = geometry {
        if let Some(window) = app_handle.get_webview_window(&window_label) {
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: geom.x,
                y: geom.y,
            }));
        }
    }

    Ok(())
}
