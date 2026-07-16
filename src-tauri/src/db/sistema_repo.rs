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
