//! Tauri application for vello benchmarks.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_benchmarks,
            commands::get_simd_levels,
            commands::run_benchmark,
            commands::save_reference,
            commands::list_references,
            commands::load_reference,
            commands::delete_reference,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
