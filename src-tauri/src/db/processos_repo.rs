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
