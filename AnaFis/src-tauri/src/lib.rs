#![warn(clippy::pedantic, clippy::nursery)]
//! `AnaFis` library crate providing core functionality for scientific computation and data management.
// Minimal modules - only what's actually used
mod data_library;
mod error;
mod export;
mod import;
pub mod scientific;
mod unit_conversion;
mod utils;
mod windows;

use tauri::{Listener, Manager};

// Startup-file state lives in its own module so that the `#[tauri::command]`
// proc-macro and `generate_handler!` don't both emit `__cmd__get_startup_file`
// into the crate-root macro namespace (which causes E0255).
pub(crate) mod startup {
    use std::sync::Mutex;
    use tauri::State;

    pub struct StartupFileState(pub Mutex<Option<String>>);

    #[tauri::command]
    #[allow(clippy::needless_pass_by_value, reason = "Tauri commands require owned State")]
    pub fn get_startup_file(
        state: State<'_, StartupFileState>,
    ) -> Result<Option<String>, String> {
        let mut file_guard = state.0.lock().map_err(|e| e.to_string())?;
        // Take the value so it is only returned once.
        Ok(file_guard.take())
    }
}

/// Main entry point for the `AnaFis` application
///
/// # Panics
///
/// Panics if the Tauri context or application cannot be initialized.
#[allow(
    clippy::too_many_lines,
    reason = "Main application setup and configuration"
)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Scientific Computation Commands
            scientific::curve_fitting::fit_custom_odr,
            scientific::curve_fitting::evaluate_model_grid,
            scientific::uncertainty_propagation::calculator::calculate_uncertainty,
            scientific::uncertainty_propagation::calculator::generate_latex,
            scientific::uncertainty_propagation::generate_uncertainty_formulas,
            // Math Function Commands (pre-compiled symb_anafis evaluators)
            // Only functions NOT natively supported by Univer
            scientific::math_functions::math_asec,
            scientific::math_functions::math_acsc,
            scientific::math_functions::math_asech,
            scientific::math_functions::math_acsch,
            scientific::math_functions::math_gamma,
            scientific::math_functions::math_digamma,
            scientific::math_functions::math_trigamma,
            scientific::math_functions::math_tetragamma,
            scientific::math_functions::math_polygamma,
            scientific::math_functions::math_beta,
            scientific::math_functions::math_zeta,
            scientific::math_functions::math_zeta_deriv,
            scientific::math_functions::math_elliptic_k,
            scientific::math_functions::math_elliptic_e,
            scientific::math_functions::math_hermite,
            scientific::math_functions::math_assoc_legendre,
            scientific::math_functions::math_spherical_harmonic,
            scientific::math_functions::math_sinc,
            scientific::math_functions::math_lambertw,
            scientific::math_functions::math_cbrt,
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
            utils::file_operations::save_binary_file,
            utils::file_operations::read_file_text,
            utils::file_operations::check_ffmpeg_available,
            utils::file_operations::transcode_webm_to_mp4,
            startup::get_startup_file,
        ])
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Load environment variables from .env file
            dotenv::dotenv().ok();

            // Initialize logging
            if let Err(e) = utils::init_logging() {
                eprintln!("Failed to initialize logging: {e}");
            }

            // Check for file association open (when app is launched with a file)
            let args: Vec<String> = std::env::args().collect();
            let mut pending_file = None;
            for arg in args.into_iter().skip(1) {
                let lower_arg = arg.to_lowercase();
                if lower_arg.ends_with(".anafispread") {
                    let normalized_path = arg.replace('\\', "/");
                    utils::log_info(&format!("Opening file from association: {normalized_path}"));
                    pending_file = Some(normalized_path);
                    break;
                }
            }
            app.manage(startup::StartupFileState(std::sync::Mutex::new(pending_file)));

            // Initialize Data Library
            match data_library::commands::init_data_library(app.handle()) {
                Ok(state) => {
                    app.manage(state);
                    utils::log_info("Data Library initialized successfully");
                }
                Err(e) => {
                    utils::log_info(&format!("WARNING: Failed to initialize Data Library: {e}"));
                }
            }

            utils::log_info(&format!("Dev mode: {}", cfg!(debug_assertions)));

            // Listen for main window events
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window("main") {
                // Force a dark native background so startup never flashes white
                // before the frontend stylesheet and React tree are ready.
                drop(main_window.set_background_color(Some(tauri::webview::Color(
                    10, 10, 10, 255,
                ))));

                // Keep hidden until frontend emits a ready event.
                drop(main_window.hide());
                let main_window_for_ready = main_window.clone();
                main_window.once("anafis://ready", move |_| {
                    drop(main_window_for_ready.show());
                    drop(main_window_for_ready.set_focus());
                });

                // Fallback: ensure main window still appears even if the ready signal is missed.
                let fallback_handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(2500));
                    if let Some(fallback_window) = fallback_handle.get_webview_window("main")
                        && matches!(fallback_window.is_visible(), Ok(false))
                    {
                        drop(fallback_window.show());
                        drop(fallback_window.set_focus());
                    }
                });

                main_window.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::Destroyed) {
                        // Main window is being destroyed, close all child windows
                        if let Some(w) = app_handle.get_webview_window("uncertainty-calculator") {
                            drop(w.close());
                        }
                        if let Some(w) = app_handle.get_webview_window("settings") {
                            drop(w.close());
                        }
                        if let Some(w) = app_handle.get_webview_window("latex-preview") {
                            drop(w.close());
                        }
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
