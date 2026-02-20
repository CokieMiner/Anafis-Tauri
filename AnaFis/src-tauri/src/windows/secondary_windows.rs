// src-tauri/src/secondary_windows.rs

use crate::windows::window_manager::{WindowConfig, create_or_focus_window};
use tauri::{AppHandle, Manager, WindowEvent};
use tokio::sync::Notify;
use tokio::time::{Duration, timeout};
use tracing::{error, info};
use urlencoding;

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn close_uncertainty_calculator_window(app: AppHandle) -> Result<(), String> {
    crate::windows::window_manager::close_window(&app, "uncertainty-calculator")
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn resize_uncertainty_calculator_window(
    app: AppHandle,
    width: f64,
    height: f64,
) -> Result<(), String> {
    crate::windows::window_manager::resize_window(&app, "uncertainty-calculator", width, height)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn open_uncertainty_calculator_window(app: AppHandle) -> Result<(), String> {
    let config = WindowConfig {
        title: "Uncertainty Calculator".to_string(),
        url: "uncertainty-calculator.html".to_string(),
        width: 600.0,  // Wider default to accommodate two-column layout properly
        height: 670.0, // Increased default height for more content
        resizable: true,
        decorations: false,
        transparent: false,
        always_on_top: true,
        skip_taskbar: true,
        parent: Some("main".to_string()),
        min_width: Some(600.0), // More reasonable minimum width for two columns
        min_height: Some(670.0), // Increased minimum height to ensure rendered formula section is always visible
        focus_on_create: true,
    };

    create_or_focus_window(&app, "uncertainty-calculator", config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn close_settings_window(app: AppHandle) -> Result<(), String> {
    crate::windows::window_manager::close_window(&app, "settings").map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn open_settings_window(app: AppHandle) -> Result<(), String> {
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
        focus_on_create: true,
    };

    create_or_focus_window(&app, "settings", config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn close_data_library_window(app: AppHandle) -> Result<(), String> {
    crate::windows::window_manager::close_window(&app, "data-library").map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn open_data_library_window(app: AppHandle) -> Result<(), String> {
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
        focus_on_create: false,
    };

    create_or_focus_window(&app, "data-library", config).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub async fn open_latex_preview_window(
    app: AppHandle,
    latex_formula: String,
    title: String,
) -> Result<(), String> {
    // Debug logging
    info!(
        "Opening LaTeX preview window with formula: {}..., title: {}",
        &latex_formula.chars().take(50).collect::<String>(),
        title
    );

    // Encode the parameters for URL
    let encoded_formula = urlencoding::encode(&latex_formula);
    let encoded_title = urlencoding::encode(&title);
    let new_url = format!("latex-preview.html?formula={encoded_formula}&title={encoded_title}");

    // Check if window already exists and destroy it
    if let Some(existing_window) = app.get_webview_window("latex-preview") {
        info!("Destroying existing LaTeX preview window");

        // Create a notify to wait for window destruction
        let notify = std::sync::Arc::new(Notify::new());
        let notify_clone = notify.clone();

        // Register the destruction listener BEFORE calling destroy()
        // This ensures the listener is active when destroy() is called
        existing_window.on_window_event(move |event| {
            if matches!(event, WindowEvent::Destroyed) {
                notify_clone.notify_one();
            }
        });

        // Create the notified future immediately after registering the listener
        // This prevents missing fast Destroyed events
        let notified_fut = notify.notified();

        // Call destroy() and handle the Result properly
        if let Err(destroy_err) = existing_window.destroy() {
            error!(
                "Failed to destroy existing LaTeX preview window: {}",
                destroy_err
            );
            return Err(format!(
                "Failed to destroy existing window: {destroy_err}"
            ));
        }

        // Wait for the window to be fully destroyed with a shorter timeout
        // Treat timeout as a hard failure to prevent race conditions
        if timeout(Duration::from_millis(500), notified_fut).await.is_ok() {
            info!("Existing LaTeX preview window destroyed successfully");
        } else {
            error!(
                "Timeout waiting for window destruction - window may not be fully destroyed"
            );
            return Err("Failed to destroy existing window: timeout waiting for destruction confirmation".to_string());
        }
    }

    info!("Creating new LaTeX preview window");
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "latex-preview",
        tauri::WebviewUrl::App(new_url.into()),
    )
    .title(&title)
    .decorations(false)
    .resizable(true)
    .inner_size(500.0_f64, 225.0_f64)
    .min_inner_size(400.0_f64, 225.0_f64)
    .max_inner_size(1600.0_f64, 225.0_f64)
    .transparent(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .closable(true)
    .build()
    .map_err(|e| {
        error!("Failed to create window: {}", e);
        format!("Failed to create window: {e}")
    })?;

    // Show and focus the window
    window.show().map_err(|e| {
        error!("Failed to show window: {}", e);
        format!("Failed to show window: {e}")
    })?;
    window.set_focus().map_err(|e| {
        error!("Failed to focus window: {}", e);
        format!("Failed to focus window: {e}")
    })?;

    info!("LaTeX preview window opened successfully");
    Ok(())
}
