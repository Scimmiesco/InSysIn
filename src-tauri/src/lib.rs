pub mod db;
pub mod hardware;
pub mod historico;
pub mod models;

use models::AppState;
use std::sync::Mutex;
use std::time::Instant;
use sysinfo::System;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // PERFORMANCE TIP: Use System::new() instead of System::new_all()
    let mut sys = System::new();

    // Coleta inicial focada apenas no necessário
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            sys: Mutex::new(sys), // Instância única mantida (Boa prática já aplicada!)
            last_db_save: Mutex::new(Instant::now()),
        })
        .invoke_handler(tauri::generate_handler![
            hardware::ler_hardware,
            historico::obter_historico
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
