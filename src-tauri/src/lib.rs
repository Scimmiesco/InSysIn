pub mod commands;
pub mod db;
pub mod models;
pub mod services;
pub mod state;

use state::AppState;
use std::sync::Mutex;
use std::time::Instant;
use sysinfo::System;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            sys: Mutex::new(sys),
            last_db_save: Mutex::new(Instant::now()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::hardware::ler_hardware,
            commands::historico::obter_historico,
            commands::historico::obter_processos_agrupados,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
