use crate::models::hardware::SysStats;
use crate::models::historico::{HistoricoCompleto, HistoricoProcesso, HistoricoSistema};
use chrono::Local;
use rusqlite::{Connection, Result};

pub fn iniciar_banco() -> Result<Connection> {
    let conn = Connection::open("historico_sistema.sqlite")?;

    // Solução: Usa query_row para ler e descartar a resposta do WAL sem gerar erros
    let _ = conn.query_row("PRAGMA journal_mode = WAL;", [], |_row| Ok(()));

    // synchronous=NORMAL não retorna linhas, então execute funciona normalmente (usamos let _ para ignorar avisos)
    let _ = conn.execute("PRAGMA synchronous = NORMAL;", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS uso_sistema (
            id INTEGER PRIMARY KEY,
            data_hora TEXT NOT NULL,
            cpu_global REAL,
            ram_usada INTEGER
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS uso_processos (
            id INTEGER PRIMARY KEY,
            data_hora TEXT NOT NULL,
            nome TEXT NOT NULL,
            cpu REAL,
            memoria INTEGER
        )",
        [],
    )?;

    Ok(conn)
}

pub fn salvar_historico(conn: &mut Connection, stats: &SysStats) -> Result<()> {
    let tx = conn.transaction()?;
    let agora = Local::now().to_rfc3339();

    tx.execute(
        "INSERT INTO uso_sistema (data_hora, cpu_global, ram_usada) VALUES (?1, ?2, ?3)",
        (&agora, stats.cpu_usage, stats.mem_info.used_memory as i64),
    )?;

    for processo in &stats.processes {
        tx.execute(
            "INSERT INTO uso_processos (data_hora, nome, cpu, memoria) VALUES (?1, ?2, ?3, ?4)",
            (
                &agora,
                &processo.name,
                processo.cpu_usage,
                processo.memory_usage as i64,
            ),
        )?;
    }

    tx.commit()?;
    Ok(())
}

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
