// src-tauri/src/window_manager.rs
use crate::error::{CommandResult, window_error};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[allow(clippy::struct_excessive_bools, reason = "Window configuration naturally involves many flags")]
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
    pub focus_on_create: bool,
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
            focus_on_create: true,
        }
    }
}

pub fn create_or_focus_window(
    app: &AppHandle,
    window_id: &str,
    config: WindowConfig,
) -> CommandResult<()> {
    // Check if window already exists
    if let Some(existing_window) = app.get_webview_window(window_id) {
        existing_window
            .show()
            .map_err(|e| window_error(e.to_string()))?;
        if config.focus_on_create {
            existing_window
                .set_focus()
                .map_err(|e| window_error(e.to_string()))?;
        }
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
        .background_color(tauri::webview::Color(0, 0, 0, 0)); // Set transparent background in builder

    // Apply minimum size constraints if specified
    if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
        builder = builder.min_inner_size(min_width, min_height);
    } else if let Some(min_width) = config.min_width {
        builder = builder.min_inner_size(min_width, config.height * 0.5);
    } else if let Some(min_height) = config.min_height {
        builder = builder.min_inner_size(config.width * 0.5, min_height);
    }

    // Set parent window if specified
    if let Some(parent_label) = &config.parent
        && let Some(parent_window) = app.get_webview_window(parent_label)
    {
        builder = builder
            .parent(&parent_window)
            .map_err(|e| window_error(format!("Failed to set parent window: {e}")))?;
    }

    let window = builder.build().map_err(|e| window_error(e.to_string()))?;

    // Ensure transparent background is set (redundant but safe)
    drop(window.set_background_color(Some(tauri::webview::Color(0, 0, 0, 0))));

    // Now show the window
    window.show().map_err(|e| window_error(e.to_string()))?;

    // Only focus if requested
    if config.focus_on_create {
        window
            .set_focus()
            .map_err(|e| window_error(e.to_string()))?;
    }

    Ok(())
}

pub fn close_window(app: &AppHandle, window_id: &str) -> CommandResult<()> {
    if let Some(window) = app.get_webview_window(window_id) {
        window.close().map_err(|e| window_error(e.to_string()))?;
        Ok(())
    } else {
        Err(window_error(format!("Window '{window_id}' not found")))
    }
}

pub fn resize_window(
    app: &AppHandle,
    window_id: &str,
    width: f64,
    height: f64,
) -> CommandResult<()> {
    if let Some(window) = app.get_webview_window(window_id) {
        window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize {
                #[allow(clippy::cast_possible_truncation, reason = "Screen coordinates fit in u32")]
                #[allow(clippy::cast_sign_loss, reason = "Window dimensions are positive")]
                width: width as u32,
                #[allow(clippy::cast_possible_truncation, reason = "Screen coordinates fit in u32")]
                #[allow(clippy::cast_sign_loss, reason = "Window dimensions are positive")]
                height: height as u32,
            }))
            .map_err(|e| window_error(e.to_string()))?;
        Ok(())
    } else {
        Err(window_error(format!("Window '{window_id}' not found")))
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn set_window_size(
    app: AppHandle,
    window_id: String,
    width: f64,
    height: f64,
) -> Result<(), String> {
    resize_window(&app, &window_id, width, height).map_err(|e| e.message)
}
