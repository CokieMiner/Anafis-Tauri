use std::sync::Mutex;
use tauri::State;

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri commands require owned State"
)]
pub fn get_startup_file(state: State<'_, StartupFileState>) -> Result<Option<String>, String> {
    let mut file_guard = state.0.lock().map_err(|e| e.to_string())?;
    // Take the value so it is only returned once.
    Ok(file_guard.take())
}

pub struct StartupFileState(pub Mutex<Option<String>>);
