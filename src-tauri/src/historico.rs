use crate::db::database::{iniciar_banco, salvar_historico};
use crate::models::historico::HistoricoCompleto;
use crate::models::historico::{HistoricoProcesso, HistoricoSistema, ProcessoAgrupado};
use rusqlite::{Connection, Result};

pub fn ler_historico(conn: &Connection) -> Result<HistoricoCompleto> {
    // Busca os últimos 60 registros do sistema (ex: últimos 60 minutos)
    let mut stmt_sys = conn.prepare(
        "SELECT data_hora, cpu_global, ram_usada FROM uso_sistema ORDER BY id DESC LIMIT 60",
    )?;
    let sistema = stmt_sys
        .query_map([], |row| {
            Ok(HistoricoSistema {
                data_hora: row.get(0)?,
                cpu_global: row.get(1)?,
                ram_usada: row.get::<_, i64>(2)? as u64, // Converte de i64 para u64
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    // Busca os últimos processos salvos
    let mut stmt_proc = conn.prepare(
        "SELECT data_hora, nome, cpu, memoria FROM uso_processos ORDER BY id DESC LIMIT 300",
    )?;
    let processos = stmt_proc
        .query_map([], |row| {
            Ok(HistoricoProcesso {
                data_hora: row.get(0)?,
                nome: row.get(1)?,
                cpu: row.get(2)?,
                memoria: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(HistoricoCompleto { sistema, processos })
}

#[tauri::command]
pub fn obter_historico() -> Result<HistoricoCompleto, String> {
    let conn = iniciar_banco().map_err(|e| e.to_string())?;

    ler_historico(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn obter_processos_agrupados(
    ordem: String,
    desc: bool,
) -> Result<Vec<ProcessoAgrupado>, String> {
    let conn = iniciar_banco().map_err(|e| e.to_string())?;

    // 1. Validação estrita para evitar SQL Injection
    let coluna_sql = match ordem.as_str() {
        "nome" => "nome",
        "data" => "ultima_data",
        "cpu" => "total_cpu",
        "memoria" => "total_memoria",
        _ => "total_memoria", // Fallback (Padrão)
    };

    let direcao_sql = if desc { "DESC" } else { "ASC" };

    // 2. Query usando GROUP BY, MAX e SUM
    let query = format!(
        "SELECT
            nome,
            MAX(data_hora) as ultima_data,
            SUM(cpu) as total_cpu,
            SUM(memoria) as total_memoria
         FROM uso_processos
         GROUP BY nome
         ORDER BY {} {}",
        coluna_sql, direcao_sql
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let processos = stmt
        .query_map([], |row| {
            Ok(ProcessoAgrupado {
                nome: row.get(0)?,
                ultima_data: row.get(1)?,
                total_cpu: row.get(2)?,
                total_memoria: row.get::<_, i64>(3)? as u64,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(processos)
}
