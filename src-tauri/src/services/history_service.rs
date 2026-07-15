use crate::models::historico::{HistoricoCompleto, HistoricoProcesso, HistoricoSistema, ProcessoAgrupado};
use rusqlite::Connection;

const LIMITE_SISTEMA: usize = 60;
const LIMITE_PROCESSOS: usize = 300;

pub fn ler_historico(conn: &Connection) -> rusqlite::Result<HistoricoCompleto> {
    let sistema = ler_sistema(conn)?;
    let processos = ler_processos(conn)?;
    Ok(HistoricoCompleto { sistema, processos })
}

fn ler_sistema(conn: &Connection) -> rusqlite::Result<Vec<HistoricoSistema>> {
    let mut stmt = conn.prepare(
        "SELECT data_hora, cpu_global, ram_usada, COALESCE(ram_total, 0) \
         FROM uso_sistema ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map([LIMITE_SISTEMA as i64], |row| {
            Ok(HistoricoSistema {
                data_hora: row.get(0)?,
                cpu_global: row.get(1)?,
                ram_usada: row.get::<_, i64>(2)? as u64,
                ram_total: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn ler_processos(conn: &Connection) -> rusqlite::Result<Vec<HistoricoProcesso>> {
    let mut stmt = conn.prepare(
        "SELECT data_hora, nome, cpu, memoria FROM uso_processos ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map([LIMITE_PROCESSOS as i64], |row| {
            Ok(HistoricoProcesso {
                data_hora: row.get(0)?,
                nome: row.get(1)?,
                cpu: row.get(2)?,
                memoria: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn listar_processos_agrupados(
    conn: &Connection,
    ordem: &str,
    desc: bool,
) -> rusqlite::Result<Vec<ProcessoAgrupado>> {
    let coluna_sql = match ordem {
        "nome" => "nome",
        "data" => "ultima_data",
        "cpu" => "total_cpu",
        "memoria" => "total_memoria",
        _ => "total_memoria",
    };
    let direcao = if desc { "DESC" } else { "ASC" };
    let query = format!(
        "SELECT nome, MAX(data_hora) as ultima_data, SUM(cpu) as total_cpu, SUM(memoria) as total_memoria \
         FROM uso_processos GROUP BY nome ORDER BY {} {}",
        coluna_sql, direcao
    );
    let mut stmt = conn.prepare(&query)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ProcessoAgrupado {
                nome: row.get(0)?,
                ultima_data: row.get(1)?,
                total_cpu: row.get(2)?,
                total_memoria: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
