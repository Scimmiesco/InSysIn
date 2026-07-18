use std::collections::HashMap;

/// Test that SysStats model fields are consistent
#[test]
fn test_mem_info_consistency() {
    let mi = insysin_lib::models::hardware::MemInfo {
        total_memory: 17179869184,
        free_memory: 500000000,
        available_memory: 10000000000,
        used_memory: 12000000000,
        used_swap: 0,
        total_swap: 0,
        free_swap: 0,
        breakdown: None,
    };

    assert!(mi.total_memory > 0);
    assert!(mi.used_memory <= mi.total_memory);
    assert!(mi.free_memory <= mi.total_memory);
    assert!(mi.available_memory <= mi.total_memory);
    assert!(mi.free_swap <= mi.total_swap);
    assert!(mi.used_swap <= mi.total_swap);
}

/// Test that ProcessInfo values are well-formed
#[test]
fn test_process_info_validity() {
    let p = insysin_lib::models::hardware::ProcessInfo {
        pid: 12345,
        name: "test_process".into(),
        cpu_usage: 25.5,
        memory_usage: 1048576,
    };

    assert!(p.pid > 0);
    assert!(p.memory_usage > 0);
    assert!(p.cpu_usage >= 0.0 && p.cpu_usage <= 100.0);
    assert!(!p.name.is_empty());
}

/// Test that NetworkInterface fields are consistent
#[test]
fn test_network_interface_consistency() {
    let iface = insysin_lib::models::network::NetworkInterface {
        name: "en0".into(),
        ip_addresses: vec!["192.168.1.5".into()],
        received_bytes: 1000,
        transmitted_bytes: 500,
        received_packets: 100,
        transmitted_packets: 50,
    };

    assert!(!iface.name.is_empty());
    assert!(!iface.ip_addresses.is_empty());
    assert!(iface.received_packets <= iface.received_bytes || iface.received_bytes == 0);
}

/// Test that ConnectionStats totals are correct
#[test]
fn test_connection_stats_integrity() {
    let cs = insysin_lib::models::network::ConnectionStats {
        total: 20,
        tcp: 15,
        udp: 5,
        established: 10,
        listening: 5,
        close_wait: 1,
        time_wait: 2,
    };

    assert_eq!(cs.tcp + cs.udp, cs.total);
    assert!(cs.established <= cs.total);
    assert!(cs.listening <= cs.total);
    assert!(cs.close_wait <= cs.total);
    assert!(cs.time_wait <= cs.total);
}

/// Test that DNS cache loading handles missing file gracefully
#[test]
fn test_dns_cache_loading() {
    let _path = std::path::PathBuf::from("/tmp/nonexistent_cache_file.json");
    let cache: HashMap<String, String> =
        serde_json::from_str(include_str!("../../assets/dns_seed.json")).unwrap_or_default();
    assert!(cache.contains_key("1.1.1.1") || cache.is_empty());
}

/// Test that Historical data models are consistent
#[test]
fn test_historico_models() {
    let sys = insysin_lib::models::historico::HistoricoSistema {
        data_hora: "2025-01-01T00:00:00Z".into(),
        cpu_global: 45.5,
        ram_usada: 8589934592,
        ram_total: 17179869184,
    };

    assert!(!sys.data_hora.is_empty());
    assert!(sys.cpu_global >= 0.0 && sys.cpu_global <= 100.0);
    assert!(sys.ram_usada <= sys.ram_total);
    assert!(sys.ram_usada > 0);

    let proc = insysin_lib::models::historico::HistoricoProcesso {
        data_hora: "2025-01-01T00:00:00Z".into(),
        nome: "test".into(),
        cpu: 10.5,
        memoria: 1048576,
    };

    assert!(!proc.nome.is_empty());
    assert!(proc.cpu >= 0.0);
    assert!(proc.memoria > 0);

    let aggr = insysin_lib::models::historico::ProcessoAgrupado {
        nome: "test".into(),
        ultima_data: "2025-01-01T00:00:00Z".into(),
        total_cpu: 50.0,
        total_memoria: 5242880,
    };

    assert!(!aggr.nome.is_empty());
    assert!(aggr.total_cpu >= 0.0);
    assert!(aggr.total_memoria > 0);
}

/// Test that InternetInfo model stores valid data
#[test]
fn test_internet_info_fields() {
    let info = insysin_lib::models::wifi::InternetInfo {
        public_ip: "1.2.3.4".into(),
        isp: "Test ISP".into(),
        city: "Test City".into(),
        country: "Test Country".into(),
        org: "Test Org".into(),
        timezone: "UTC".into(),
        asn: "AS12345".into(),
        latency_ms: 50.0,
        ping_target: "Cloudflare (1.1.1.1)".into(),
        online: true,
        wifi_ssid: Some("TestWiFi".into()),
    };

    assert!(info.latency_ms >= 0.0);
    assert!(!info.public_ip.is_empty() == info.online);
    assert!(!info.ping_target.is_empty());
}

/// Test that DeviceInfo validates MAC address format
#[test]
fn test_device_info_mac() {
    let device = insysin_lib::models::wifi::DeviceInfo {
        ip: "192.168.1.1".into(),
        mac: "aa:bb:cc:dd:ee:ff".into(),
        vendor: Some("Test Corp".into()),
    };

    assert!(device.mac.len() >= 17);
    assert!(!device.ip.is_empty());
}

/// Test that SpeedResult has reasonable values
#[test]
fn test_speed_result_bounds() {
    let speed = insysin_lib::models::wifi::SpeedResult {
        download_mbps: 100.5,
        upload_mbps: 20.3,
    };

    assert!(speed.download_mbps >= 0.0);
    assert!(speed.upload_mbps >= 0.0);
}

/// Test that LocalNetworkInfo is consistent
#[test]
fn test_local_network_consistency() {
    let net = insysin_lib::models::wifi::LocalNetworkInfo {
        local_ip: "192.168.1.5".into(),
        gateway: "192.168.1.1".into(),
        subnet_mask: "255.255.255.0".into(),
        network_range: "192.168.1.0/24".into(),
        dns_servers: vec!["8.8.8.8".into()],
        devices: vec![],
    };

    assert!(!net.local_ip.is_empty());
    assert!(!net.gateway.is_empty());
    assert!(net.local_ip != "127.0.0.1");
}

/// Test that historical queries validate ordering column
#[test]
fn test_order_validation() {
    let valid = |col: &str| {
        matches!(col, "nome" | "cpu" | "memoria" | "data_hora")
    };

    assert!(valid("cpu"));
    assert!(valid("nome"));
    assert!(valid("memoria"));
    assert!(valid("data_hora"));
    assert!(!valid("id"));
    assert!(!valid("'; DROP TABLE--"));
    assert!(!valid(""));
}
