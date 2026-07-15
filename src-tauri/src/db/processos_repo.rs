use crate::models::historico::{HistoricoProcesso, ProcessoAgrupado};
use chrono::Local;
use rusqlite::{Connection, Result};

pub fn inserir(conn: &Connection, nome: &str, cpu: f32, memoria: u64) -> Result<()> {
    let agora = Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO uso_processos (data_hora, nome, cpu, memoria) VALUES (?1, ?2, ?3, ?4)",
        (&agora, nome, cpu, memoria as i64),
    )?;
    Ok(())
}

pub fn listar_ultimos(conn: &Connection, limite: usize) -> Result<Vec<HistoricoProcesso>> {
    let mut stmt = conn.prepare(
        "SELECT data_hora, nome, cpu, memoria FROM uso_processos ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map([limite as i64], |row| {
            Ok(HistoricoProcesso {
                data_hora: row.get(0)?,
                nome: row.get(1)?,
                cpu: row.get(2)?,
                memoria: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn listar_agrupados(
    conn: &Connection,
    ordem: &str,
    desc: bool,
) -> Result<Vec<ProcessoAgrupado>> {
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
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}
