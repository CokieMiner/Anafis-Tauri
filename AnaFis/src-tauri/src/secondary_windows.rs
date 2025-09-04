// src-tauri/src/secondary_windows.rs

use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn close_uncertainty_calculator_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("uncertainty-calculator") {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        Ok(())
    } else {
        Err("Uncertainty calculator window not found".to_string())
    }
}

#[tauri::command]
pub fn resize_uncertainty_calculator_window(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("uncertainty-calculator") {
        window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: width as u32,
            height: height as u32,
        })).map_err(|e| format!("Failed to resize window: {}", e))?;
        Ok(())
    } else {
        Err("Uncertainty calculator window not found".to_string())
    }
}

#[tauri::command]
pub async fn open_uncertainty_calculator_window(app: AppHandle) -> Result<(), String> {
    // First check if window already exists
    if let Some(existing_window) = app.get_webview_window("uncertainty-calculator") {
        existing_window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        return Ok(());
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "uncertainty-calculator",
        tauri::WebviewUrl::App("uncertainty-calculator.html".into())
    )
    .title("Uncertainty Calculator")
    .decorations(false)
    // Make the window non-resizable so the native frame can't be interactively resized
    .resizable(false)
    // Match the content size exactly (slightly larger width to cover edge artifacts) and keep the WebView transparent to avoid a white flash
    .inner_size(504.0_f64, 450.0_f64)
    .transparent(true)
    .closable(true)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;

    // Show and focus the window
    // Make the native background dark immediately to avoid any white flash on some platforms
    let _ = window.set_background_color(Some(tauri::webview::Color(10, 10, 10, 255)));
    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn close_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
        Ok(())
    } else {
        Err("Settings window not found".to_string())
    }
}

#[tauri::command]
pub fn resize_settings_window(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: width as u32,
            height: height as u32,
        })).map_err(|e| format!("Failed to resize window: {}", e))?;
        Ok(())
    } else {
        Err("Settings window not found".to_string())
    }
}

#[tauri::command]
pub async fn open_settings_window(app: AppHandle) -> Result<(), String> {
    // First check if window already exists
    if let Some(existing_window) = app.get_webview_window("settings") {
        existing_window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        return Ok(());
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into())
    )
    .title("AnaFis Settings")
    .decorations(false)
    // Make the window non-resizable so the native frame can't be interactively resized
    .resizable(false)
    // Match the content size exactly and keep the WebView transparent to avoid a white flash
    .inner_size(650.0_f64, 700.0_f64)
    .transparent(true)
    .closable(true)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;

    // Show and focus the window
    // Make the native background dark immediately to avoid any white flash on some platforms
    let _ = window.set_background_color(Some(tauri::webview::Color(10, 10, 10, 255)));
    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn close_latex_preview_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("latex-preview") {
        window.destroy().map_err(|e| format!("Failed to destroy window: {}", e))?;
        Ok(())
    } else {
        Err("LaTeX preview window not found".to_string())
    }
}

#[tauri::command]
pub async fn open_latex_preview_window(app: AppHandle, latex_formula: String, title: String) -> Result<(), String> {
    // First check if window already exists
    if let Some(existing_window) = app.get_webview_window("latex-preview") {
        existing_window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        return Ok(());
    }

    // Encode the parameters for URL
    let encoded_formula = urlencoding::encode(&latex_formula);
    let encoded_title = urlencoding::encode(&title);
    let url = format!("latex-preview.html?formula={}&title={}", encoded_formula, encoded_title);

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
    .closable(true)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;

    // Show and focus the window
    let _ = window.set_background_color(Some(tauri::webview::Color(10, 10, 10, 255)));
    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;

    Ok(())
}
