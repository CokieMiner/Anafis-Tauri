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
    let window = WebviewWindowBuilder::new(app, window_id, WebviewUrl::App(config.url.into()))
        .title(&config.title)
        .inner_size(config.width, config.height)
        .resizable(config.resizable)
        .decorations(config.decorations)
        .transparent(config.transparent)
        .always_on_top(config.always_on_top)
        .closable(true)
        .build()
        .map_err(|e| AnaFisError::Window(e.to_string()))?;

    // Set dark background to prevent white flash
    let _ = window.set_background_color(Some(tauri::webview::Color(10, 10, 10, 255)));

    window.show().map_err(|e| AnaFisError::Window(e.to_string()))?;
    window.set_focus().map_err(|e| AnaFisError::Window(e.to_string()))?;

    Ok(())
}

pub fn close_window(app: &AppHandle, window_id: &str) -> Result<(), AnaFisError> {
    if let Some(window) = app.get_webview_window(window_id) {
        window.close().map_err(|e| AnaFisError::Window(e.to_string()))?;
        Ok(())
    } else {
        Err(AnaFisError::Window(format!("Window '{}' not found", window_id)))
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
        Err(AnaFisError::Window(format!("Window '{}' not found", window_id)))
    }
}
