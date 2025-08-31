// src-tauri/src/tabs.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, WebviewWindowBuilder, Emitter};
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::task;
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

    pub async fn start_ipc_server_static(app_handle: AppHandle) {
        Self::handle_ipc_connections(app_handle).await;
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

    async fn handle_ipc_connections(app_handle: AppHandle) {
        let addr = "127.0.0.1:0"; // Let OS assign a port

        match TcpListener::bind(addr).await {
            Ok(listener) => {
                let local_addr = listener.local_addr().unwrap();
                println!("IPC server started on {}", local_addr);

                // Store the port for clients to connect
                // For simplicity, we'll use a fixed port
                let port = 12345;
                let addr = format!("127.0.0.1:{}", port);

                match TcpListener::bind(&addr).await {
                    Ok(listener) => {
                        loop {
                            match listener.accept().await {
                                Ok((mut stream, _)) => {
                                    let app_handle = app_handle.clone();
                                    task::spawn(async move {
                                        let mut buffer = [0; 1024];
                                        match stream.read(&mut buffer).await {
                                            Ok(n) if n > 0 => {
                                                let data = &buffer[..n];
                                                if let Ok(tab_info_str) = std::str::from_utf8(data) {
                                                    if let Ok(tab_info) = serde_json::from_str::<TabInfo>(tab_info_str.trim()) {
                                                        // Create new window with received tab
                                                        let manager = TAB_MANAGER.lock().await;
                                                        let _ = manager.create_tab_window(&app_handle, tab_info, None).await;
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    });
                                }
                                Err(e) => {
                                    println!("IPC connection error: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to bind IPC socket: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to bind IPC socket: {}", e);
            }
        }
    }    pub async fn create_tab_window(
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

        let mut builder = WebviewWindowBuilder::new(
            app_handle,
            &window_id,
            tauri::WebviewUrl::App(format!("index.html?detached=true&tabId={}&tabType={}&tabTitle={}",
                tab_info.id, tab_info.content_type, urlencoding::encode(&tab_info.title)).into())
        )
        .title(&tab_info.title)
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

    pub async fn close_tab_window(&self, app_handle: &AppHandle, window_id: &str) -> Result<(), String> {
        if let Some(window) = app_handle.get_webview_window(window_id) {
            window.close().map_err(|e| format!("Failed to close window: {}", e))?;

            let mut windows = self.windows.lock().await;
            windows.remove(window_id);

            Ok(())
        } else {
            Err("Window not found".to_string())
        }
    }

    pub async fn get_window_state(&self, window_id: &str) -> Option<WindowState> {
        let windows = self.windows.lock().await;
        windows.get(window_id).cloned()
    }

    pub async fn save_state(&self) -> Result<(), String> {
        let windows = self.windows.lock().await;
        let state_json = serde_json::to_string(&*windows)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        // Save to file or use Tauri's store
        std::fs::write("tab_state.json", state_json)
            .map_err(|e| format!("Failed to save state: {}", e))?;

        Ok(())
    }

    pub async fn load_state(&self) -> Result<(), String> {
        if let Ok(state_json) = std::fs::read_to_string("tab_state.json") {
            let windows: HashMap<String, WindowState> = serde_json::from_str(&state_json)
                .map_err(|e| format!("Failed to deserialize state: {}", e))?;

            let mut current_windows = self.windows.lock().await;
            *current_windows = windows;
        }

        Ok(())
    }
}

static TAB_MANAGER: Lazy<Arc<Mutex<TabManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(TabManager::new()))
});

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[tauri::command]
pub async fn create_tab_window(
    app_handle: AppHandle,
    tab_info: TabInfo,
    geometry: Option<WindowGeometry>,
    position: Option<WindowPosition>,
) -> Result<String, String> {
    let manager = TAB_MANAGER.lock().await;
    let final_geometry = if position.is_some() && geometry.is_none() {
        Some(WindowGeometry {
            x: position.as_ref().unwrap().x,
            y: position.as_ref().unwrap().y,
            width: 800,
            height: 600,
        })
    } else {
        geometry
    };

    manager.create_tab_window(&app_handle, tab_info, final_geometry).await
}

#[tauri::command]
pub async fn close_tab_window(app_handle: AppHandle, window_id: String) -> Result<(), String> {
    let manager = TAB_MANAGER.lock().await;
    manager.close_tab_window(&app_handle, &window_id).await
}

#[tauri::command]
pub async fn get_window_state(window_id: String) -> Result<WindowState, String> {
    let manager = TAB_MANAGER.lock().await;
    manager.get_window_state(&window_id).await
        .ok_or_else(|| "Window state not found".to_string())
}

#[tauri::command]
pub async fn save_tab_state() -> Result<(), String> {
    let manager = TAB_MANAGER.lock().await;
    manager.save_state().await
}

#[tauri::command]
pub async fn load_tab_state() -> Result<(), String> {
    let manager = TAB_MANAGER.lock().await;
    manager.load_state().await
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
