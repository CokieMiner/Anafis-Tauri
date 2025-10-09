// src-tauri/src/window_manager.rs
use tauri::{AppHandle, WebviewWindowBuilder, Manager, WebviewUrl};
use crate::utils::error::AnaFisError;

pub struct WindowConfig {
    pub title: String,
    pub url: String,
    pub width: f64,
    pub height: f64,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub always_on_top: bool,
    pub skip_taskbar: bool,
    pub parent: Option<String>,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "AnaFis Window".to_string(),
            url: "index.html".to_string(),
            width: 800.0,
            height: 600.0,
            resizable: true,
            decorations: false,
            transparent: true,
            always_on_top: false,
            skip_taskbar: true,
            parent: Some("main".to_string()),
            min_width: None,
            min_height: None,
        }
    }
}

pub fn create_or_focus_window(
    app: &AppHandle,
    window_id: &str,
    config: WindowConfig
) -> Result<(), AnaFisError> {
    // Check if window already exists
    if let Some(existing_window) = app.get_webview_window(window_id) {
        existing_window.show().map_err(|e| AnaFisError::Window(e.to_string()))?;
        existing_window.set_focus().map_err(|e| AnaFisError::Window(e.to_string()))?;
        return Ok(());
    }

    // Create new window
    let mut builder = WebviewWindowBuilder::new(app, window_id, WebviewUrl::App(config.url.into()))
        .title(&config.title)
        .inner_size(config.width, config.height)
        .resizable(config.resizable)
        .decorations(config.decorations)
        .transparent(config.transparent)
        .always_on_top(config.always_on_top)
        .skip_taskbar(config.skip_taskbar)
        .center()
        .focused(false) // Don't focus initially
        .closable(true)
        .visible(false) // Initially hidden to prevent white flash
        .background_color(tauri::webview::Color(10, 10, 10, 255)); // Set dark background in builder
    
    // Apply minimum size constraints if specified
    if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
        builder = builder.min_inner_size(min_width, min_height);
    } else if let Some(min_width) = config.min_width {
        builder = builder.min_inner_size(min_width, config.height * 0.5);
    } else if let Some(min_height) = config.min_height {
        builder = builder.min_inner_size(config.width * 0.5, min_height);
    }
    
    // Set parent window if specified
    if let Some(parent_label) = &config.parent {
        if let Some(parent_window) = app.get_webview_window(parent_label) {
            builder = builder.parent(&parent_window)
                .map_err(|e| AnaFisError::Window(format!("Failed to set parent window: {}", e)))?;
        }
    }
    
    let window = builder.build()
        .map_err(|e| AnaFisError::Window(e.to_string()))?;

    // Ensure dark background is set (redundant but safe)
    let _ = window.set_background_color(Some(tauri::webview::Color(10, 10, 10, 255)));

    // Small delay to ensure webview has loaded with dark background
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Now show and focus the window
    window.show().map_err(|e| AnaFisError::Window(e.to_string()))?;
    window.set_focus().map_err(|e| AnaFisError::Window(e.to_string()))?;

    Ok(())
}

pub fn close_window(app: &AppHandle, window_id: &str) -> Result<(), AnaFisError> {
    if let Some(window) = app.get_webview_window(window_id) {
        window.close().map_err(|e| AnaFisError::Window(e.to_string()))?;
        Ok(())
    } else {
        Err(AnaFisError::Window(format!("Window '{window_id}' not found")))
    }
}

pub fn resize_window(app: &AppHandle, window_id: &str, width: f64, height: f64) -> Result<(), AnaFisError> {
    if let Some(window) = app.get_webview_window(window_id) {
        window.set_size(tauri::Size::Physical(
            tauri::PhysicalSize {
                width: width as u32,
                height: height as u32,
            }
        )).map_err(|e| AnaFisError::Window(e.to_string()))?;
        Ok(())
    } else {
        Err(AnaFisError::Window(format!("Window '{window_id}' not found")))
    }
}
