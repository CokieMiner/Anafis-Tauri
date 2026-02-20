#![warn(clippy::pedantic, clippy::nursery)]
//! Main entry point for the `AnaFis` application (Tauri entry point).

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    anafis_lib::run();
}
