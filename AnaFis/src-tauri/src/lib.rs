mod uncertainty;
mod secondary_windows;
mod tabs;

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
      secondary_windows::resize_settings_window,
      tabs::create_tab_window,
      tabs::send_tab_to_main,
      tabs::ensure_home_tab
    ])
    .setup(|app| {
      // Ensure home tab exists
      let app_handle_home = app.handle().clone();
      let _ = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
          let _ = tabs::ensure_home_tab(app_handle_home);
        });
      });

      // Get main window for event handling
      let main_window = app.get_webview_window("main").unwrap();

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
            // Main window is being destroyed, save state and close all child windows
            if let Some(calc_window) = app_handle.get_webview_window("uncertainty-calculator") {
              let _ = calc_window.close();
            }

            if let Some(settings_window) = app_handle.get_webview_window("settings") {
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