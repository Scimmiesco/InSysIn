use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tauri::AppHandle;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct WsRequest {
    command: String,
    #[serde(default)]
    args: Value,
}

#[derive(Debug, Serialize)]
struct WsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn dispatch(app: &AppHandle, req: &WsRequest) -> Result<Value, String> {
    match req.command.as_str() {
        "ler_hardware" => {
            let result = crate::commands::hardware::ler_hardware(app.clone()).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "ler_rede" => {
            let result = crate::commands::network::ler_rede(app.clone()).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_internet_info" => {
            let result = crate::commands::wifi::get_internet_info().await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "run_speed_test" => {
            let result = crate::commands::wifi::run_speed_test().await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "get_local_network_info" => {
            let result = crate::commands::wifi::get_local_network_info().await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "obter_historico" => {
            let result = crate::commands::historico::obter_historico().await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "obter_processos_agrupados" => {
            let ordem = req.args["ordem"].as_str().unwrap_or("cpu").to_string();
            let desc = req.args["desc"].as_bool().unwrap_or(true);
            let result = crate::commands::historico::obter_processos_agrupados(ordem, desc).await?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        other => Err(format!("Unknown command: {}", other)),
    }
}

async fn handle_connection(stream: TcpStream, app: AppHandle) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[WS] accept error: {}", e);
            return;
        }
    };
    println!("[WS] client connected");

    let (mut write, mut read) = ws.split();

    while let Some(Ok(msg)) = read.next().await {
        match msg {
            Message::Text(text) => {
                let req: WsRequest = match serde_json::from_str(&text) {
                    Ok(r) => r,
                    Err(e) => {
                        let resp = WsResponse {
                            result: None,
                            error: Some(format!("Parse error: {}", e)),
                        };
                        if let Ok(json) = serde_json::to_string(&resp) {
                            let _ = write.send(Message::Text(json.into())).await;
                        }
                        continue;
                    }
                };

                let response = match dispatch(&app, &req).await {
                    Ok(val) => WsResponse {
                        result: Some(val),
                        error: None,
                    },
                    Err(e) => WsResponse {
                        result: None,
                        error: Some(e),
                    },
                };

                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = write.send(Message::Text(json.into())).await;
                }
            }
            Message::Close(_) => {
                println!("[WS] client disconnected");
                break;
            }
            _ => {}
        }
    }
}

pub async fn start(app: AppHandle, port: u16) {
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[WS] bind failed on port {}: {}", port, e);
            return;
        }
    };
    println!("[WS] bridge listening on 0.0.0.0:{}", port);

    while let Ok((stream, _addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, app.clone()));
    }
}
