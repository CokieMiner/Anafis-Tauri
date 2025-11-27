// Minimal modules - only what's actually used
mod data_library;
mod error;
mod export;
mod import;
pub mod scientific;
mod uncertainty_calculator;
mod unit_conversion;
mod utils;
mod windows;

use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Uncertainty Calculator Commands (2 commands)
            uncertainty_calculator::uncertainty::calculate_uncertainty,
            uncertainty_calculator::uncertainty::generate_latex,

            // Unit Conversion Commands (12 commands)
            unit_conversion::commands::convert_value,
            unit_conversion::commands::get_conversion_preview,
            unit_conversion::commands::check_unit_compatibility,
            unit_conversion::commands::get_available_units,
            unit_conversion::commands::quick_convert_value,
            unit_conversion::commands::get_conversion_factor,
            unit_conversion::commands::parse_unit_formula,
            unit_conversion::commands::analyze_dimensional_compatibility,
            unit_conversion::commands::get_unit_dimensional_formula,
            unit_conversion::commands::validate_unit_string,
            unit_conversion::commands::get_supported_categories,

            // Window Management Commands (9 commands)
            windows::secondary_windows::open_latex_preview_window,
            windows::secondary_windows::open_uncertainty_calculator_window,
            windows::secondary_windows::close_uncertainty_calculator_window,
            windows::secondary_windows::resize_uncertainty_calculator_window,
            windows::secondary_windows::open_settings_window,
            windows::secondary_windows::close_settings_window,
            windows::secondary_windows::open_data_library_window,
            windows::secondary_windows::close_data_library_window,
            windows::window_manager::set_window_size,

            // Scientific Computation Commands (Sidebar tools)
            scientific::uncertainty_propagation::generate_uncertainty_formulas

            // Data Library Commands (12 commands)
            data_library::commands::save_sequence,
            data_library::commands::get_sequences,
            data_library::commands::get_sequence,
            data_library::commands::update_sequence,
            data_library::commands::delete_sequence,
            data_library::commands::get_sequence_stats,
            data_library::commands::pin_sequence,
            data_library::commands::duplicate_sequence,
            data_library::commands::get_all_tags,
            data_library::commands::export_sequences_csv,
            data_library::commands::batch_import_sequences,

            // Export Commands (2 commands - dispatcher + snapshot)
            export::export_data,
            export::anafispread::export_anafispread,

            // Import Commands (3 commands)
            import::import_spreadsheet_file,
            import::import_anafis_spread_direct,
            import::get_file_metadata,

            // Utility Commands (File Operations)
            utils::file_operations::save_png_file,
            utils::file_operations::save_image_from_data_url,
            utils::file_operations::save_svg_file,
        ])
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize logging
            if let Err(e) = utils::init_logging() {
                eprintln!("Failed to initialize logging: {e}");
            }

            // Check for file association open (when app is launched with a file)
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let file_path = args[1].clone();
                if file_path.ends_with(".anafispread") {
                    utils::log_info(&format!("Opening file from association: {}", file_path));
                    // We'll emit an event to the frontend to handle the file opening
                    let app_handle = app.handle().clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.emit("open-file", file_path);
                        }
                    });
                }
            }

            // Initialize Data Library
            match data_library::commands::init_data_library(app.handle()) {
                Ok(state) => {
                    app.manage(state);
                    utils::log_info("Data Library initialized successfully");
                }
                Err(e) => {
                    utils::log_info(&format!(
                        "WARNING: Failed to initialize Data Library: {}",
                        e
                    ));
                }
            }

            utils::log_info("Using system Python - no embedded Python setup needed");
            utils::log_info(&format!("Dev mode: {}", cfg!(debug_assertions)));

            // Check if Python is available in PATH
            let current_path = std::env::var("PATH").unwrap_or_default();
            let has_python = current_path.split(';').any(|path| {
                let python_path = std::path::Path::new(path).join("python.exe");
                python_path.exists()
            });

            if has_python {
                utils::log_info("SUCCESS: Python found in system PATH");
            } else {
                utils::log_info("WARNING: Python not found in system PATH - PyO3 may fail");
            }

            // Don't set PYTHONHOME or PYTHONPATH - let PyO3 use system Python
            // Remove any existing Python environment variables that might interfere
            std::env::remove_var("PYTHONHOME");
            std::env::remove_var("PYTHONPATH");
            std::env::remove_var("PYO3_PYTHON");

            utils::log_info("Environment setup complete - using system Python");

            // Listen for main window events
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window("main") {
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::Focused(true) => {
                            // Main window gained focus - no action needed
                        }
                        tauri::WindowEvent::Destroyed => {
                            // Main window is being destroyed, close all child windows
                            let _ = app_handle
                                .get_webview_window("uncertainty-calculator")
                                .and_then(|w| w.close().ok());
                            let _ = app_handle
                                .get_webview_window("settings")
                                .and_then(|w| w.close().ok());
                            let _ = app_handle
                                .get_webview_window("latex-preview")
                                .and_then(|w| w.close().ok());
                        }
                        _ => {}
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
