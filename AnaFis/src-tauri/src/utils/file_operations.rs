// File operations utilities

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

fn ensure_parent_and_write(path: &str, content: impl AsRef<[u8]>) -> Result<(), String> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {e}"))?;
    }

    fs::write(path, content).map_err(|e| format!("Failed to write file: {e}"))?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct FfmpegAvailability {
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VideoExportResult {
    pub output_path: String,
    pub output_format: String,
    pub warning: Option<String>,
}

fn find_ffmpeg_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    let locator = ("where", "ffmpeg");

    #[cfg(not(target_os = "windows"))]
    let locator = ("which", "ffmpeg");

    let output = Command::new(locator.0).arg(locator.1).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let path_text = String::from_utf8(output.stdout).ok()?;
    path_text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

/// Read a text file and return its contents as a String.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn read_file_text(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read file '{path}': {e}"))
}

/// Save a PNG file from base64-encoded data
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn save_png_file(path: String, data: String) -> Result<(), String> {
    // Decode base64 data
    let bytes = general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| format!("Failed to decode base64 data: {e}"))?;

    ensure_parent_and_write(&path, bytes)
}

/// Save an image file from a data URL (format: "data:image/png;base64,...")
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn save_image_from_data_url(data_url: String, path: String) -> Result<(), String> {
    // Split the data URL to extract base64 data
    let parts: Vec<&str> = data_url.split(',').collect();
    if parts.len() != 2 {
        return Err(
            "Invalid data URL format. Expected 'data:image/[type];base64,[data]'".to_string(),
        );
    }

    let base64_data = parts[1];

    // Decode base64 data
    let bytes = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64 data: {e}"))?;

    ensure_parent_and_write(&path, bytes)
}

/// Save an SVG file from SVG content string
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn save_svg_file(svg_content: String, path: String) -> Result<(), String> {
    ensure_parent_and_write(&path, svg_content)
}

/// Save a generic binary file from base64-encoded data.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn save_binary_file(path: String, data_base64: String) -> Result<(), String> {
    let bytes = general_purpose::STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Failed to decode base64 data: {e}"))?;

    ensure_parent_and_write(&path, bytes)
}

/// Check whether `FFmpeg` is available in the current machine.
#[tauri::command]
pub fn check_ffmpeg_available() -> FfmpegAvailability {
    let path = find_ffmpeg_path();
    let ffmpeg_bin = path.as_deref().unwrap_or("ffmpeg");

    let output = Command::new(ffmpeg_bin).arg("-version").output();
    match output {
        Ok(output) if output.status.success() => {
            let version_line = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("ffmpeg")
                .to_string();

            FfmpegAvailability {
                available: true,
                version: Some(version_line),
                path,
                message: None,
            }
        }
        _ => FfmpegAvailability {
            available: false,
            version: None,
            path: None,
            message: Some(
                "FFmpeg not found. MP4 export is unavailable; WebM export remains available."
                    .to_string(),
            ),
        },
    }
}

/// Transcode a `WebM` file to MP4 using `FFmpeg`.
/// Falls back to returning the input `WebM` path when transcoding is unavailable/fails.
#[tauri::command]
#[allow(clippy::needless_pass_by_value, reason = "Tauri command")]
pub fn transcode_webm_to_mp4(
    input_webm_path: String,
    output_mp4_path: String,
) -> Result<VideoExportResult, String> {
    if !Path::new(&input_webm_path).exists() {
        return Err(format!("Input WebM file does not exist: {input_webm_path}"));
    }

    let ffmpeg_status = check_ffmpeg_available();
    if !ffmpeg_status.available {
        return Ok(VideoExportResult {
            output_path: input_webm_path,
            output_format: "webm".to_string(),
            warning: ffmpeg_status.message,
        });
    }

    if let Some(parent) = Path::new(&output_mp4_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory for MP4 export: {e}"))?;
    }

    let ffmpeg_bin = ffmpeg_status.path.as_deref().unwrap_or("ffmpeg");
    let output = Command::new(ffmpeg_bin)
        .args([
            "-y",
            "-i",
            &input_webm_path,
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            &output_mp4_path,
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(VideoExportResult {
            output_path: output_mp4_path,
            output_format: "mp4".to_string(),
            warning: None,
        }),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(VideoExportResult {
                output_path: input_webm_path,
                output_format: "webm".to_string(),
                warning: Some(format!(
                    "MP4 transcoding failed. Falling back to WebM. FFmpeg output: {}",
                    stderr.trim()
                )),
            })
        }
        Err(error) => Ok(VideoExportResult {
            output_path: input_webm_path,
            output_format: "webm".to_string(),
            warning: Some(format!(
                "Failed to launch FFmpeg. Falling back to WebM: {error}"
            )),
        }),
    }
}
