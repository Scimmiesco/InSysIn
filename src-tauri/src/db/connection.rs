use rusqlite::{Connection, Result};

pub fn abrir_conexao() -> Result<Connection> {
    let conn = Connection::open("historico_sistema.sqlite")?;

    let _ = conn.query_row("PRAGMA journal_mode = WAL;", [], |_row| Ok(()));
    let _ = conn.execute("PRAGMA synchronous = NORMAL;", []);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS uso_sistema (
            id INTEGER PRIMARY KEY,
            data_hora TEXT NOT NULL,
            cpu_global REAL,
            ram_usada INTEGER,
            ram_total INTEGER
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

    let _ = conn.execute("ALTER TABLE uso_sistema ADD COLUMN ram_total INTEGER", []);

    Ok(conn)
}
