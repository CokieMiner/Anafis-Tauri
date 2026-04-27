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

use std::env::args;
mod startup;

use std::sync::Mutex;
use std::thread::{sleep, spawn};
use std::time::Duration;

use tauri::webview::Color;
use tauri::{Builder, Listener, Manager, WindowEvent, generate_context, generate_handler};
use tauri_plugin_dialog::init;

use crate::data_library::commands as data_commands;
use crate::export::anafispread::export_anafispread;
use crate::export::export_data;
use crate::import::{get_file_metadata, import_anafis_spread_direct, import_spreadsheet_file};
use crate::scientific::curve_fitting::commands as curve_commands;
use crate::scientific::math_functions as math_commands;
use crate::scientific::uncertainty_propagation::calculator as uncertainty_calc;
use crate::scientific::uncertainty_propagation::generate_uncertainty_formulas;
use crate::unit_conversion::commands as unit_commands;
use crate::utils::file_operations as file_ops;
use crate::utils::{init_logging, log_info};
use crate::windows::secondary_windows as window_commands;
use crate::windows::window_manager as manager_commands;
use dotenv::dotenv;

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
#[allow(
    clippy::exit,
    clippy::print_stderr,
    reason = "Main application entry point and logging fallback"
)]
pub fn run() {
    Builder::default()
        .invoke_handler(generate_handler![
            // Scientific Computation Commands
            curve_commands::fit_custom_odr,
            curve_commands::evaluate_model_curve,
            curve_commands::evaluate_model_grid,
            uncertainty_calc::calculate_uncertainty,
            uncertainty_calc::generate_latex,
            generate_uncertainty_formulas,
            // Math Function Commands (pre-compiled symb_anafis evaluators)
            // Only functions NOT natively supported by Univer
            math_commands::math_asec,
            math_commands::math_acsc,
            math_commands::math_asech,
            math_commands::math_acsch,
            math_commands::math_gamma,
            math_commands::math_digamma,
            math_commands::math_trigamma,
            math_commands::math_tetragamma,
            math_commands::math_polygamma,
            math_commands::math_beta,
            math_commands::math_zeta,
            math_commands::math_zeta_deriv,
            math_commands::math_elliptic_k,
            math_commands::math_elliptic_e,
            math_commands::math_hermite,
            math_commands::math_assoc_legendre,
            math_commands::math_spherical_harmonic,
            math_commands::math_sinc,
            math_commands::math_lambertw,
            math_commands::math_cbrt,
            // Unit Conversion Commands (12 commands)
            unit_commands::convert_value,
            unit_commands::get_conversion_preview,
            unit_commands::check_unit_compatibility,
            unit_commands::get_available_units,
            unit_commands::quick_convert_value,
            unit_commands::get_conversion_factor,
            unit_commands::parse_unit_formula,
            unit_commands::analyze_dimensional_compatibility,
            unit_commands::get_unit_dimensional_formula,
            unit_commands::validate_unit_string,
            unit_commands::get_supported_categories,
            // Window Management Commands (9 commands)
            window_commands::open_latex_preview_window,
            window_commands::open_uncertainty_calculator_window,
            window_commands::close_uncertainty_calculator_window,
            window_commands::resize_uncertainty_calculator_window,
            window_commands::open_settings_window,
            window_commands::close_settings_window,
            window_commands::open_data_library_window,
            window_commands::close_data_library_window,
            manager_commands::set_window_size,
            // Data Library Commands (12 commands)
            data_commands::save_sequence,
            data_commands::get_sequences,
            data_commands::get_sequence,
            data_commands::update_sequence,
            data_commands::delete_sequence,
            data_commands::get_sequence_stats,
            data_commands::pin_sequence,
            data_commands::duplicate_sequence,
            data_commands::get_all_tags,
            data_commands::export_sequences_csv,
            data_commands::batch_import_sequences,
            // Export Commands (2 commands - dispatcher + snapshot)
            export_data,
            export_anafispread,
            // Import Commands (3 commands)
            import_spreadsheet_file,
            import_anafis_spread_direct,
            get_file_metadata,
            // Utility Commands (File Operations)
            file_ops::save_png_file,
            file_ops::save_image_from_data_url,
            file_ops::save_svg_file,
            file_ops::save_binary_file,
            file_ops::read_file_text,
            file_ops::check_ffmpeg_available,
            file_ops::transcode_webm_to_mp4,
            startup::get_startup_file,
        ])
        .plugin(init())
        .setup(|app| {
            // Load environment variables from .env file
            dotenv().ok();

            // Initialize logging
            if let Err(e) = init_logging() {
                eprintln!("Failed to initialize logging: {e}");
            }

            // Check for file association open (when app is launched with a file)
            let args: Vec<String> = args().collect();
            let mut pending_file = None;
            for arg in args.into_iter().skip(1) {
                let lower_arg = arg.to_lowercase();
                if lower_arg.ends_with(".anafispread") {
                    let normalized_path = arg.replace('\\', "/");
                    log_info(&format!("Opening file from association: {normalized_path}"));
                    pending_file = Some(normalized_path);
                    break;
                }
            }
            app.manage(startup::StartupFileState(Mutex::new(pending_file)));

            // Initialize Data Library
            match data_commands::init_data_library(app.handle()) {
                Ok(state) => {
                    app.manage(state);
                    log_info("Data Library initialized successfully");
                }
                Err(e) => {
                    log_info(&format!("WARNING: Failed to initialize Data Library: {e}"));
                }
            }

            log_info(&format!("Dev mode: {}", cfg!(debug_assertions)));

            // Listen for main window events
            let app_handle = app.handle().clone();
            if let Some(main_window) = app.get_webview_window("main") {
                // Force a dark native background so startup never flashes white
                // before the frontend stylesheet and React tree are ready.
                drop(main_window.set_background_color(Some(Color(10, 10, 10, 255))));

                // Keep hidden until frontend emits a ready event.
                drop(main_window.hide());
                let main_window_for_ready = main_window.clone();
                main_window.once("anafis://ready", move |_| {
                    drop(main_window_for_ready.show());
                    drop(main_window_for_ready.set_focus());
                });

                // Fallback: ensure main window still appears even if the ready signal is missed.
                let fallback_handle = app.handle().clone();
                spawn(move || {
                    sleep(Duration::from_millis(2500));
                    if let Some(fallback_window) = fallback_handle.get_webview_window("main")
                        && matches!(fallback_window.is_visible(), Ok(false))
                    {
                        drop(fallback_window.show());
                        drop(fallback_window.set_focus());
                    }
                });

                main_window.on_window_event(move |event| {
                    if matches!(event, WindowEvent::Destroyed) {
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
        .run(generate_context!())
        .expect("error while running tauri application");
}
