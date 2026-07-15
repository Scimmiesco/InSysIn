use crate::db::{connection, processos_repo, sistema_repo};
use crate::models::hardware::SysStats;
use rusqlite::Connection;

pub fn salvar_snapshot(stats: &SysStats) -> Result<(), String> {
    let conn = connection::abrir_conexao().map_err(|e| e.to_string())?;
    salvar_snapshot_com_cte(&conn, stats)
}

pub fn salvar_snapshot_com_cte(conn: &Connection, stats: &SysStats) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    sistema_repo::inserir(
        &tx,
        stats.cpu_usage,
        stats.mem_info.used_memory,
        stats.mem_info.total_memory,
    )
    .map_err(|e| e.to_string())?;

    for p in &stats.processes {
        processos_repo::inserir(&tx, &p.name, p.cpu_usage, p.memory_usage)
            .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
