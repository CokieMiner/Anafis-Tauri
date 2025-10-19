// Minimal modules - only what's actually used
mod uncertainty_calculator;
mod windows;
mod utils;
mod unit_conversion;
mod scientific;
mod data_library;
mod export;

use tauri::Manager;

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
            windows::tabs::send_tab_to_main,
            windows::tabs::create_tab_window,

            // Scientific Computation Commands (Sidebar tools)
            scientific::uncertainty_propagation::generate_uncertainty_formulas,

            // Data Library Commands (11 commands)
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
            data_library::commands::export_sequences_json,
            
            // Export Commands (8 commands)
            export::export_data,
            export::text::export_to_text,
            export::json::export_to_json,
            export::excel::export_to_excel,
            export::html::export_to_html,
            export::markdown::export_to_markdown,
            export::tex::export_to_latex,
            export::parquet::export_to_parquet,
            
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

            // Initialize Data Library
            match data_library::commands::init_data_library(app.handle()) {
                Ok(state) => {
                    app.manage(state);
                    utils::log_info("Data Library initialized successfully");
                }
                Err(e) => {
                    utils::log_info(&format!("WARNING: Failed to initialize Data Library: {}", e));
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
            if let Some(main_window) = app.get_window("main") {
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::Focused(true) => {
                            // Main window gained focus - no action needed
                        }
                        tauri::WindowEvent::Destroyed => {
                            // Main window is being destroyed, close all child windows
                            let _ = app_handle.get_webview_window("uncertainty-calculator")
                                .and_then(|w| w.close().ok());
                            let _ = app_handle.get_webview_window("settings")
                                .and_then(|w| w.close().ok());
                            let _ = app_handle.get_webview_window("latex-preview")
                                .and_then(|w| w.close().ok());

                            // Close all detached tab windows (they start with "tab_")
                            let windows = app_handle.webview_windows();
                            for (label, window) in windows {
                                if label.starts_with("tab_") {
                                    let _ = window.close();
                                }
                            }
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
