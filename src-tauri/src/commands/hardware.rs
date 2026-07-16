use crate::services::{db_service, hardware_service};
use crate::state::AppState;
use std::time::Instant;
use tauri::State;

const SAVE_INTERVAL_SECS: u64 = 180;

#[tauri::command]
pub fn ler_hardware(state: State<'_, AppState>) -> crate::models::hardware::SysStats {
    let mut sys = state.sys.lock().expect("sys mutex poisoned");
    let mut disks = state.disks.lock().expect("disks mutex poisoned");
    let mut networks = state.networks.lock().expect("networks mutex poisoned");
    let stats = hardware_service::coletar_dados(&mut sys, &mut disks, &mut networks);

    let mut last_save = state.last_db_save.lock().expect("last_db_save mutex poisoned");

    if last_save.elapsed().as_secs() >= SAVE_INTERVAL_SECS {
        match db_service::salvar_snapshot(&stats) {
            Ok(()) => println!("Dados salvos no SQLite com sucesso!"),
            Err(e) => eprintln!("Erro ao salvar no banco: {}", e),
        }
        *last_save = Instant::now();
    }

    stats
}
