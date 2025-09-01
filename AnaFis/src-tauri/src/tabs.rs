// src-tauri/src/tabs.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewWindowBuilder, Emitter};
use tokio::sync::Mutex;

use once_cell::sync::Lazy;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub content_type: String,
    pub state: serde_json::Value,
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowState {
    pub tabs: Vec<TabInfo>,
    pub geometry: WindowGeometry,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct TabManager {
    windows: Mutex<HashMap<String, WindowState>>,
    home_tab_id: String,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            home_tab_id: "home".to_string(),
        }
    }

    pub async fn ensure_home_tab(&self, app_handle: &AppHandle) -> Result<(), String> {
        let window_id = format!("tab_{}", self.home_tab_id);

        if app_handle.get_webview_window(&window_id).is_some() {
            return Ok(());
        }

        let home_tab = TabInfo {
            id: self.home_tab_id.clone(),
            title: "AnaFis Home".to_string(),
            content_type: "home".to_string(),
            state: serde_json::json!({"welcome": "Welcome to AnaFis"}),
            icon: None,
        };

        self.create_tab_window(app_handle, home_tab, None).await?;
        Ok(())
    }

    pub async fn create_tab_window(
        &self,
        app_handle: &AppHandle,
        tab_info: TabInfo,
        geometry: Option<WindowGeometry>,
    ) -> Result<String, String> {
        let window_id = format!("tab_{}", tab_info.id);

        // Check if window already exists
        if app_handle.get_webview_window(&window_id).is_some() {
            return Err("Window already exists".to_string());
        }

        let title_with_icon = match tab_info.content_type.as_str() {
            "home" => format!("🏠 {}", tab_info.title),
            "spreadsheet" => format!("📊 {}", tab_info.title),
            "fitting" => format!("📈 {}", tab_info.title),
            "solver" => format!("🧮 {}", tab_info.title),
            "montecarlo" => format!("🎲 {}", tab_info.title),
            _ => format!("🏠 {}", tab_info.title),
        };

        let mut builder = WebviewWindowBuilder::new(
            app_handle,
            &window_id,
            tauri::WebviewUrl::App(format!("index.html?detached=true&tabId={}&tabType={}&tabTitle={}",
                tab_info.id, tab_info.content_type, urlencoding::encode(&tab_info.title)).into())
        )
        .title(&title_with_icon)
        .decorations(false)  // Disable native decorations to use custom title bar
        .resizable(true)
        .closable(true)
        .transparent(true);  // Enable transparency for better custom title bar integration

        if let Some(ref geom) = geometry {
            builder = builder
                .inner_size(geom.width as f64, geom.height as f64)
                .position(geom.x as f64, geom.y as f64);
        } else {
            // Adjust height to account for custom title bar (approximately 32px)
            builder = builder.inner_size(800.0, 600.0 + 32.0);
        }

        let window = builder
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;

        // Set dark background to prevent white flash
        let _ = window.set_background_color(Some(tauri::webview::Color(10, 10, 10, 255)));

        // Store the tab state
        let mut windows = self.windows.lock().await;
        windows.insert(window_id.clone(), WindowState {
            tabs: vec![tab_info],
            geometry: geometry.unwrap_or(WindowGeometry {
                x: 100,
                y: 100,
                width: 800,
                height: 600,
            }),
        });

        Ok(window_id)
    }




}

static TAB_MANAGER: Lazy<Arc<Mutex<TabManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(TabManager::new()))
});

#[tauri::command]
pub async fn create_tab_window(
    app_handle: AppHandle,
    tab_info: TabInfo,
    geometry: Option<WindowGeometry>,
) -> Result<String, String> {
    let manager = TAB_MANAGER.lock().await;
    manager.create_tab_window(&app_handle, tab_info, geometry).await
}



#[tauri::command]
pub async fn ensure_home_tab(app_handle: AppHandle) -> Result<(), String> {
    let manager = TAB_MANAGER.lock().await;
    manager.ensure_home_tab(&app_handle).await
}

#[tauri::command]
pub async fn send_tab_to_main(
    app_handle: AppHandle,
    tab_id: String,
    tab_type: String,
    tab_title: String
) -> Result<(), String> {
    // Find the main window
    let main_window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    // Send the tab data to the main window via IPC
    let tab_info = TabInfo {
        id: tab_id,
        title: tab_title,
        content_type: tab_type,
        state: serde_json::json!({"reattached": true}),
        icon: None,
    };

    // Emit an event to the main window to reattach the tab
    main_window
        .emit("reattach-tab", &tab_info)
        .map_err(|e| format!("Failed to emit reattach event: {}", e))?;

    Ok(())
}
