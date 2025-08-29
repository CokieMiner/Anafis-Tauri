mod uncertainty;
mod secondary_windows;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      uncertainty::calculate_uncertainty,
      secondary_windows::open_uncertainty_calculator_window,
      secondary_windows::close_uncertainty_calculator_window,
      secondary_windows::resize_uncertainty_calculator_window,
      secondary_windows::open_settings_window,
      secondary_windows::close_settings_window,
      secondary_windows::resize_settings_window
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Set dark background for main window to prevent white flash
      let main_window = app.get_webview_window("main").unwrap();
      // With transparent WebView, we rely on HTML/CSS background instead
      // main_window.set_background_color(Some(tauri::webview::Color(10, 10, 10, 255)))?;

      // Listen for main window events
      let app_handle = app.handle().clone();
      main_window.on_window_event(move |event| {
        match event {
          tauri::WindowEvent::Focused(true) => {
            // Main window gained focus, bring calculator and settings windows to front if they exist
            if let Some(calc_window) = app_handle.get_webview_window("uncertainty-calculator") {
              let _ = calc_window.set_focus();
            }
            if let Some(settings_window) = app_handle.get_webview_window("settings") {
              let _ = settings_window.set_focus();
            }
          }
          tauri::WindowEvent::Destroyed => {
            // Main window is being destroyed, close all child windows
            println!("Main window destroyed, closing all child windows...");

            if let Some(calc_window) = app_handle.get_webview_window("uncertainty-calculator") {
              println!("Closing uncertainty calculator window");
              let _ = calc_window.close();
            }

            if let Some(settings_window) = app_handle.get_webview_window("settings") {
              println!("Closing settings window");
              let _ = settings_window.close();
            }
          }
          _ => {}
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}