// src-tauri/src/secondary_windows.rs

use tauri::{AppHandle, Manager};
use pyo3::prelude::*;
use crate::windows::window_manager::{create_or_focus_window, WindowConfig};

#[tauri::command]
pub fn close_uncertainty_calculator_window(app: AppHandle) -> Result<(), String> {
    crate::windows::window_manager::close_window(&app, "uncertainty-calculator")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resize_uncertainty_calculator_window(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    crate::windows::window_manager::resize_window(&app, "uncertainty-calculator", width, height)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_uncertainty_calculator_window(app: AppHandle) -> Result<(), String> {
    // First check if window already exists
    if let Some(existing_window) = app.get_webview_window("uncertainty-calculator") {
        existing_window.show().map_err(|e| format!("Failed to show window: {e}"))?;
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {e}"))?;
        return Ok(());
    }

    // Initialize Python module when opening the uncertainty calculator window
    // This ensures Python is ready when the first calculation is made
    let app_clone = app.clone();
    std::thread::spawn(move || {
        let _ = Python::attach(|py| -> PyResult<()> {
            crate::uncertainty_calculator::initialize_python_module(py, &app_clone)
        });
    });

    let config = WindowConfig {
        title: "Uncertainty Calculator".to_string(),
        url: "uncertainty-calculator.html".to_string(),
        width: 600.0, // Wider default to accommodate two-column layout properly
        height: 670.0, // Increased default height for more content
        resizable: true,
        decorations: false,
        transparent: false,
        always_on_top: true,
        skip_taskbar: true,
        parent: Some("main".to_string()),
        min_width: Some(600.0), // More reasonable minimum width for two columns
        min_height: Some(670.0), // Increased minimum height to ensure rendered formula section is always visible
    };

    create_or_focus_window(&app, "uncertainty-calculator", config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_settings_window(app: AppHandle) -> Result<(), String> {
    crate::windows::window_manager::close_window(&app, "settings")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_settings_window(app: AppHandle) -> Result<(), String> {
    // First check if window already exists
    if let Some(existing_window) = app.get_webview_window("settings") {
        existing_window.show().map_err(|e| format!("Failed to show window: {e}"))?;
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {e}"))?;
        return Ok(());
    }

    let config = WindowConfig {
        title: "AnaFis Settings".to_string(),
        url: "settings.html".to_string(),
        width: 650.0,
        height: 700.0,
        resizable: true,
        decorations: false,
        transparent: true,
        always_on_top: true,
        skip_taskbar: true,
        parent: Some("main".to_string()),
        min_width: Some(500.0),
        min_height: Some(500.0),
    };

    create_or_focus_window(&app, "settings", config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_data_library_window(app: AppHandle) -> Result<(), String> {
    crate::windows::window_manager::close_window(&app, "data-library")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_data_library_window(app: AppHandle) -> Result<(), String> {
    // First check if window already exists
    if let Some(existing_window) = app.get_webview_window("data-library") {
        existing_window.show().map_err(|e| format!("Failed to show window: {e}"))?;
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {e}"))?;
        return Ok(());
    }

    let config = WindowConfig {
        title: "Data Library".to_string(),
        url: "data-library.html".to_string(),
        width: 1000.0,
        height: 700.0,
        resizable: true,
        decorations: false,
        transparent: true,
        always_on_top: true,
        skip_taskbar: true,
        parent: Some("main".to_string()),
        min_width: Some(700.0),
        min_height: Some(500.0),
    };

    create_or_focus_window(&app, "data-library", config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_latex_preview_window(app: AppHandle, latex_formula: String, title: String) -> Result<(), String> {
    // First check if window already exists
    if let Some(existing_window) = app.get_webview_window("latex-preview") {
        existing_window.show().map_err(|e| format!("Failed to show window: {e}"))?;
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {e}"))?;
        return Ok(());
    }

    // Encode the parameters for URL
    let encoded_formula = urlencoding::encode(&latex_formula);
    let encoded_title = urlencoding::encode(&title);
    let url = format!("latex-preview.html?formula={encoded_formula}&title={encoded_title}");

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "latex-preview",
        tauri::WebviewUrl::App(url.into())
    )
    .title(&title)
    .decorations(false)
    .resizable(true)
    .inner_size(500.0_f64, 225.0_f64)
    .min_inner_size(400.0_f64, 225.0_f64)
    .max_inner_size(1600.0_f64, 225.0_f64)
    .transparent(true)
    .skip_taskbar(true)
    .closable(true)
    .build()
    .map_err(|e| format!("Failed to create window: {e}"))?;

    // Show and focus the window
    let _ = window.set_background_color(Some(tauri::webview::Color(0, 0, 0, 0)));
    window.show().map_err(|e| format!("Failed to show window: {e}"))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {e}"))?;

    Ok(())
}
