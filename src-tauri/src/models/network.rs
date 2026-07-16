use serde::Serialize;

#[derive(Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub received_packets: u64,
    pub transmitted_packets: u64,
}

#[derive(Serialize)]
pub struct NetConnection {
    pub pid: i32,
    pub process_name: String,
    pub protocol: String,
    pub local: String,
    pub remote: String,
    pub state: String,
    pub hostname: Option<String>,
}

#[derive(Serialize)]
pub struct TrafficSummary {
    pub total_received: u64,
    pub total_transmitted: u64,
    pub physical: Vec<NetworkInterface>,
    pub virtual_ifaces: Vec<NetworkInterface>,
    pub special: Vec<NetworkInterface>,
}

#[derive(Serialize)]
pub struct ConnectionStats {
    pub total: usize,
    pub tcp: usize,
    pub udp: usize,
    pub established: usize,
    pub listening: usize,
    pub close_wait: usize,
    pub time_wait: usize,
}

#[derive(Serialize)]
pub struct ProcessConnection {
    pub process_name: String,
    pub count: usize,
    pub tcp_count: usize,
    pub udp_count: usize,
}

#[derive(Serialize)]
pub struct ListeningService {
    pub process_name: String,
    pub pid: i32,
    pub protocol: String,
    pub local: String,
    pub port_desc: String,
}

#[derive(Serialize)]
pub struct NetworkDashboard {
    pub traffic: TrafficSummary,
    pub connections: Vec<NetConnection>,
    pub stats: ConnectionStats,
    pub top_processes: Vec<ProcessConnection>,
    pub listening_services: Vec<ListeningService>,
}
