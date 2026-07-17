use crate::db::connection;
use crate::services::history_service;

#[tauri::command]
pub async fn obter_historico() -> Result<crate::models::historico::HistoricoCompleto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = connection::abrir_conexao().map_err(|e| e.to_string())?;
        history_service::ler_historico(&conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn obter_processos_agrupados(
    ordem: String,
    desc: bool,
) -> Result<Vec<crate::models::historico::ProcessoAgrupado>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let conn = connection::abrir_conexao().map_err(|e| e.to_string())?;
        history_service::listar_processos_agrupados(&conn, &ordem, desc).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
