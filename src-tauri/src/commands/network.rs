use crate::models::network::{
    ConnectionStats, ListeningService, NetConnection, NetworkDashboard, NetworkInterface,
    ProcessConnection, TrafficSummary,
};
use crate::state::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn ler_rede(state: State<'_, AppState>) -> Result<NetworkDashboard, String> {
    let networks = state.networks.lock().map_err(|e| format!("networks lock: {}", e))?;

    let mut physical = Vec::new();
    let mut virtual_ifaces = Vec::new();
    let mut special = Vec::new();
    let mut total_received = 0u64;
    let mut total_transmitted = 0u64;

    for (name, data) in &*networks {
        let ips: Vec<String> = data
            .ip_networks()
            .iter()
            .map(|ip| ip.addr.to_string())
            .collect();
        let rx = data.received();
        let tx = data.transmitted();
        total_received += rx;
        total_transmitted += tx;

        let iface = NetworkInterface {
            name: name.clone(),
            ip_addresses: ips,
            received_bytes: rx,
            transmitted_bytes: tx,
            received_packets: data.total_packets_received(),
            transmitted_packets: data.total_packets_transmitted(),
        };

        let n = name.to_lowercase();
        if n.starts_with("en") {
            physical.push(iface);
        } else if n.starts_with("utun")
            || n.starts_with("bridge")
            || n.starts_with("gif")
            || n.starts_with("stf")
            || n.starts_with("vnic")
            || n.starts_with("anpi")
        {
            virtual_ifaces.push(iface);
        } else {
            special.push(iface);
        }
    }
    drop(networks);

    let connections = get_connections().unwrap_or_default();
    let stats = compute_stats(&connections);
    let top_processes = compute_top_processes(&connections);
    let listening_services = compute_listening_services(&connections);

    Ok(NetworkDashboard {
        traffic: TrafficSummary {
            total_received,
            total_transmitted,
            physical,
            virtual_ifaces,
            special,
        },
        connections,
        stats,
        top_processes,
        listening_services,
    })
}

fn compute_stats(connections: &[NetConnection]) -> ConnectionStats {
    let mut stats = ConnectionStats {
        total: 0,
        tcp: 0,
        udp: 0,
        established: 0,
        listening: 0,
        close_wait: 0,
        time_wait: 0,
    };
    for conn in connections {
        stats.total += 1;
        match conn.protocol.as_str() {
            "TCP" => stats.tcp += 1,
            "UDP" => stats.udp += 1,
            _ => {}
        }
        match conn.state.as_str() {
            "ESTABLISHED" => stats.established += 1,
            "LISTEN" => stats.listening += 1,
            "CLOSE_WAIT" => stats.close_wait += 1,
            "TIME_WAIT" => stats.time_wait += 1,
            _ => {}
        }
    }
    stats
}

fn compute_top_processes(connections: &[NetConnection]) -> Vec<ProcessConnection> {
    let mut map: HashMap<String, ProcessConnection> = HashMap::new();
    for conn in connections {
        let entry = map.entry(conn.process_name.clone()).or_insert(ProcessConnection {
            process_name: conn.process_name.clone(),
            count: 0,
            tcp_count: 0,
            udp_count: 0,
        });
        entry.count += 1;
        match conn.protocol.as_str() {
            "TCP" => entry.tcp_count += 1,
            "UDP" => entry.udp_count += 1,
            _ => {}
        }
    }
    let mut result: Vec<ProcessConnection> = map.into_values().collect();
    result.sort_by(|a, b| b.count.cmp(&a.count));
    result.truncate(10);
    result
}

fn compute_listening_services(connections: &[NetConnection]) -> Vec<ListeningService> {
    let mut services: Vec<ListeningService> = Vec::new();
    for conn in connections {
        if conn.state == "LISTEN" && conn.protocol == "TCP" {
            let port_desc = port_desc(&conn.local);
            services.push(ListeningService {
                process_name: conn.process_name.clone(),
                pid: conn.pid,
                protocol: conn.protocol.clone(),
                local: conn.local.clone(),
                port_desc,
            });
        }
    }
    services.sort_by(|a, b| a.port_desc.cmp(&b.port_desc));
    services
}

fn port_desc(addr: &str) -> String {
    let port = addr
        .split(':')
        .last()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(0);
    match port {
        80 => "HTTP".to_string(),
        443 => "HTTPS".to_string(),
        22 => "SSH".to_string(),
        53 => "DNS".to_string(),
        25 => "SMTP".to_string(),
        993 => "IMAPS".to_string(),
        3306 => "MySQL".to_string(),
        5432 => "PostgreSQL".to_string(),
        6379 => "Redis".to_string(),
        8080 => "HTTP-alt".to_string(),
        8443 => "HTTPS-alt".to_string(),
        5353 => "mDNS".to_string(),
        123 => "NTP".to_string(),
        _ => format!(":{}", port),
    }
}

fn get_connections() -> Result<Vec<NetConnection>, String> {
    let output = std::process::Command::new("lsof")
        .args(["-i", "-P", "-n"])
        .output()
        .map_err(|e| format!("failed to run lsof: {}", e))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut connections = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let pid: i32 = parts[1].parse().unwrap_or(0);
        let process_name = parts[0].to_string();
        let protocol = parts[7].to_string();

        let name_str = parts[8..].join(" ");

        let state;
        let addr;
        if let Some(paren_start) = name_str.rfind('(') {
            if name_str.ends_with(')') {
                state = name_str[paren_start + 1..name_str.len() - 1].to_string();
                addr = name_str[..paren_start].trim().to_string();
            } else {
                state = String::new();
                addr = name_str.clone();
            }
        } else {
            state = String::new();
            addr = name_str.clone();
        }

        let (local, remote) = if let Some(arrow) = addr.find("->") {
            (addr[..arrow].to_string(), addr[arrow + 2..].to_string())
        } else {
            (addr, String::new())
        };

        connections.push(NetConnection {
            pid,
            process_name,
            protocol,
            local,
            remote,
            state,
        });
    }

    Ok(connections)
}
