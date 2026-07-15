use crate::models::historico::HistoricoSistema;
use chrono::Local;
use rusqlite::{Connection, Result};

pub fn inserir(conn: &Connection, cpu_global: f32, ram_usada: u64, ram_total: u64) -> Result<()> {
    let agora = Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO uso_sistema (data_hora, cpu_global, ram_usada, ram_total) VALUES (?1, ?2, ?3, ?4)",
        (&agora, cpu_global, ram_usada as i64, ram_total as i64),
    )?;
    Ok(())
}

pub fn listar_ultimos(conn: &Connection, limite: usize) -> Result<Vec<HistoricoSistema>> {
    let mut stmt = conn.prepare(
        "SELECT data_hora, cpu_global, ram_usada, COALESCE(ram_total, 0) FROM uso_sistema ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map([limite as i64], |row| {
            Ok(HistoricoSistema {
                data_hora: row.get(0)?,
                cpu_global: row.get(1)?,
                ram_usada: row.get::<_, i64>(2)? as u64,
                ram_total: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}
