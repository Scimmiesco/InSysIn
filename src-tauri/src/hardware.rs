use crate::db::database::{iniciar_banco, salvar_historico};
use crate::models::hardware::{MemInfo, ProcessInfo, SysStats};
use crate::models::AppState;
use rusqlite::{Connection, Result};
use std::cmp::Ord;
use std::collections::HashMap;
use sysinfo::ProcessesToUpdate;
use tauri::State;

#[tauri::command]
pub fn ler_hardware(state: State<'_, AppState>) -> SysStats {
    let mut sys = state.sys.lock().unwrap();

    // PERFORMANCE TIP: Refresh ONLY what you need, not the whole system.
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut grouped_processes: HashMap<String, ProcessInfo> = HashMap::new();

    for (pid, process) in sys.processes() {
        let name = process
            .exe()
            .and_then(|path| path.file_name())
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_else(|| process.name().to_string_lossy().to_string());

        let entry = grouped_processes
            .entry(name.clone())
            .or_insert(ProcessInfo {
                pid: pid.as_u32(), // Mantém o PID do primeiro processo do grupo
                name,
                cpu_usage: 0.0,
                memory_usage: 0,
            });

        // Soma os valores dos processos com o mesmo nome
        entry.cpu_usage += process.cpu_usage();
        entry.memory_usage = entry.memory_usage.max(process.memory());
    }

    let mut consolidated_processes: Vec<ProcessInfo> = grouped_processes.into_values().collect();
    let limit = 15;

    if consolidated_processes.len() > limit {
        consolidated_processes.select_nth_unstable_by(limit - 1, |a, b| {
            b.memory_usage
                .partial_cmp(&a.memory_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Corta tudo que passou do limite
        consolidated_processes.truncate(limit);
    }

    consolidated_processes.sort_unstable_by(|a, b| {
        b.memory_usage
            .partial_cmp(&a.memory_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let sys_stats = SysStats {
        mem_info: MemInfo {
            total_memory: sys.total_memory(),
            free_memory: sys.free_memory(),
            available_memory: sys.available_memory(),
            used_memory: sys.used_memory(),
            used_swap: sys.used_swap(),
            total_swap: sys.total_swap(),
            free_swap: sys.free_swap(),
        },
        cpu_usage: sys.global_cpu_usage(),
        processes: consolidated_processes,
    };

    let mut last_save = state.last_db_save.lock().unwrap();

    if last_save.elapsed().as_secs() >= 180 {
        match iniciar_banco() {
            Ok(mut conn) => {
                // Em vez de 'let _', capturamos e mostramos o erro no terminal
                if let Err(e) = salvar_historico(&mut conn, &sys_stats) {
                    eprintln!("Erro ao salvar no banco: {}", e);
                } else {
                    println!("Dados salvos no SQLite com sucesso!"); // Confirmação visual
                }
            }
            Err(e) => eprintln!("Erro ao iniciar banco: {}", e),
        }

        *last_save = std::time::Instant::now();
    }

    return sys_stats;
}
