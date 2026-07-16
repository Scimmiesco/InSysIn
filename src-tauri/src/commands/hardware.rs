use crate::services::{db_service, hardware_service};
use crate::state::AppState;
use std::time::Instant;
use tauri::State;

const SAVE_INTERVAL_SECS: u64 = 180;

#[tauri::command]
pub fn ler_hardware(state: State<'_, AppState>) -> Result<crate::models::hardware::SysStats, String> {
    let mut sys = state.sys.lock().map_err(|e| format!("sys lock: {}", e))?;
    let mut disks = state.disks.lock().map_err(|e| format!("disks lock: {}", e))?;
    let mut networks = state.networks.lock().map_err(|e| format!("networks lock: {}", e))?;
    let stats = hardware_service::coletar_dados(&mut sys, &mut disks, &mut networks);

    let mut last_save = state.last_db_save.lock().map_err(|e| format!("last_save lock: {}", e))?;

    if last_save.elapsed().as_secs() >= SAVE_INTERVAL_SECS {
        match db_service::salvar_snapshot(&stats) {
            Ok(()) => println!("Dados salvos no SQLite com sucesso!"),
            Err(e) => eprintln!("Erro ao salvar no banco: {}", e),
        }
        *last_save = Instant::now();
    }

    Ok(stats)
}
