use crate::models::wifi::{
    DeviceInfo, InternetDiagnostics, InternetInfo, LocalNetworkInfo, SpeedResult,
};
use std::time::Instant;

fn fetch_json(url: &str) -> Result<serde_json::Value, String> {
    let resp = ureq::get(url)
        .set("User-Agent", "InSysIn/0.1.0")
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("request failed: {}", e))?;
    resp.into_json()
        .map_err(|e| format!("parse failed: {}", e))
}

fn measure_latency(target: &str) -> f64 {
    let total = Instant::now();
    let mut latencies: Vec<f64> = Vec::new();
    for _ in 0..4 {
        let start = Instant::now();
        if ureq::get(target)
            .set("User-Agent", "InSysIn/0.1.0")
            .timeout(std::time::Duration::from_secs(5))
            .call()
            .is_ok()
        {
            latencies.push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    let _ = total.elapsed();
    if latencies.is_empty() {
        return 0.0;
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    latencies[latencies.len() / 2]
}

fn download_speed() -> Result<f64, String> {
    let url = "https://speed.cloudflare.com/__down?bytes=1000000";
    let start = Instant::now();
    let mut bytes = 0u64;
    let resp = ureq::get(url)
        .set("User-Agent", "InSysIn/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .call()
        .map_err(|e| format!("download failed: {}", e))?;
    let mut reader = resp.into_reader();
    let mut buf = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut reader, &mut buf)
            .map_err(|e| format!("read failed: {}", e))?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
    }
    let elapsed = start.elapsed().as_secs_f64();
    if elapsed > 0.0 && bytes > 0 {
        let bits = (bytes as f64) * 8.0;
        Ok(bits / elapsed / 1_000_000.0)
    } else {
        Err("speed calculation failed".into())
    }
}

fn upload_speed() -> Result<f64, String> {
    let url = "https://speed.cloudflare.com/__up";
    let data = vec![0u8; 500_000];
    let start = Instant::now();
    let _resp = ureq::post(url)
        .set("User-Agent", "InSysIn/0.1.0")
        .set("Content-Type", "application/octet-stream")
        .timeout(std::time::Duration::from_secs(30))
        .send_bytes(&data)
        .map_err(|e| format!("upload failed: {}", e))?;
    let elapsed = start.elapsed().as_secs_f64();
    if elapsed > 0.0 {
        let bits = (data.len() as f64) * 8.0;
        Ok(bits / elapsed / 1_000_000.0)
    } else {
        Err("speed calculation failed".into())
    }
}

fn get_wifi_ssid() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("ipconfig")
            .args(["getsummary", "en0"])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            if let Some(rest) = line.trim().strip_prefix("SSID : ") {
                let v = rest.trim();
                if !v.is_empty() && !v.starts_with('<') {
                    return Some(v.to_string());
                }
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        let out = std::process::Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            if let Some(ssid) = line.trim().strip_prefix("SSID") {
                let v = ssid.trim_start_matches(&[' ', ':', '\t']).trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    {
        let out = std::process::Command::new("iwgetid")
            .arg("-r")
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    { None }
}

#[tauri::command]
pub async fn get_internet_info() -> Result<InternetDiagnostics, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut online = false;
        let mut public_ip = String::new();
        let mut isp = String::new();
        let mut city = String::new();
        let mut country = String::new();
        let mut org = String::new();
        let mut timezone = String::new();
        let mut asn_str = String::new();
        let mut latency_ms = 0.0f64;

        let ssid = get_wifi_ssid();

        let ip_resp = fetch_json("https://ipapi.co/json/");
        match ip_resp {
            Ok(v) => {
                online = true;
                public_ip = v.get("ip").and_then(|x| x.as_str()).unwrap_or("").to_string();
                isp = v.get("org").and_then(|x| x.as_str()).unwrap_or("").to_string();
                city = v.get("city").and_then(|x| x.as_str()).unwrap_or("").to_string();
                country = v
                    .get("country_name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                org = v.get("org").and_then(|x| x.as_str()).unwrap_or("").to_string();
                timezone = v.get("timezone").and_then(|x| x.as_str()).unwrap_or("").to_string();
                asn_str = v
                    .get("asn")
                    .and_then(|x| x.as_str())
                    .or_else(|| v.get("asn").and_then(|x| x.as_str()))
                    .unwrap_or("")
                    .to_string();
            }
            Err(_) => {
                if let Ok(ip) = fetch_json("https://api.ipify.org?format=json") {
                    online = true;
                    public_ip = ip
                        .get("ip")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            }
        }

        if online {
            latency_ms = measure_latency("https://1.1.1.1");
        }

        Ok(InternetDiagnostics {
            info: InternetInfo {
                public_ip,
                isp,
                city,
                country,
                org,
                timezone,
                asn: asn_str,
                latency_ms: (latency_ms * 10.0).round() / 10.0,
                ping_target: "Cloudflare (1.1.1.1)".into(),
                online,
                wifi_ssid: ssid,
            },
            speed: None,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn run_speed_test() -> Result<SpeedResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let down = download_speed()?;
        let up = upload_speed()?;
        Ok(SpeedResult {
            download_mbps: (down * 10.0).round() / 10.0,
            upload_mbps: (up * 10.0).round() / 10.0,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn lookup_vendor(mac: &str) -> Option<String> {
    let mac = mac.to_uppercase().replace(':', "-").replace(' ', "");
    if mac.is_empty() || mac.len() < 17 {
        return None;
    }
    let url = format!("https://api.macvendors.com/{}", mac);
    ureq::get(&url)
        .set("User-Agent", "InSysIn/0.1.0")
        .timeout(std::time::Duration::from_millis(1500))
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn batch_lookup_vendors(macs: &[String]) -> Vec<Option<String>> {
    if macs.is_empty() {
        return vec![];
    }
    let results: Vec<std::sync::Mutex<Option<String>>> = (0..macs.len())
        .map(|_| std::sync::Mutex::new(None))
        .collect();
    std::thread::scope(|s| {
        for (i, mac) in macs.iter().enumerate() {
            let mac = mac.clone();
            let result = &results[i];
            s.spawn(move || {
                *result.lock().unwrap() = lookup_vendor(&mac);
            });
        }
    });
    results
        .into_iter()
        .map(|m| m.into_inner().unwrap())
        .collect()
}

fn calc_network_range(ip: &str, mask: &str) -> String {
    let ip_parts: Vec<u8> = ip.split('.').filter_map(|s| s.parse().ok()).collect();
    let mask_parts: Vec<u8> = mask.split('.').filter_map(|s| s.parse().ok()).collect();
    if ip_parts.len() != 4 || mask_parts.len() != 4 {
        return String::new();
    }
    let mut network = [0u8; 4];
    for i in 0..4 {
        network[i] = ip_parts[i] & mask_parts[i];
    }
    let prefix_len = mask_parts
        .iter()
        .map(|&o| o.count_ones())
        .sum::<u32>();
    format!(
        "{}.{}.{}.{}/{}",
        network[0], network[1], network[2], network[3], prefix_len
    )
}

fn calc_broadcast(ip: &str, mask: &str) -> String {
    let ip_parts: Vec<u8> = ip.split('.').filter_map(|s| s.parse().ok()).collect();
    let mask_parts: Vec<u8> = mask.split('.').filter_map(|s| s.parse().ok()).collect();
    if ip_parts.len() != 4 || mask_parts.len() != 4 {
        return String::new();
    }
    let b: Vec<u8> = (0..4)
        .map(|i| ip_parts[i] | (!mask_parts[i]))
        .collect();
    format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3])
}

fn warm_arp_table(broadcast: &str) {
    if broadcast.is_empty() || broadcast == "0.0.0.0" || broadcast == "255.255.255.255" {
        return;
    }
    let _ = std::process::Command::new("ping")
        .args(["-c", "1", "-W", "500", "-t", "1", broadcast])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output();
    std::thread::sleep(std::time::Duration::from_millis(1500));
}

#[cfg(target_os = "macos")]
fn get_local_network() -> Result<LocalNetworkInfo, String> {
    use std::process::Command;

    let iface = find_active_iface()?;
    let ifconfig = Command::new("ifconfig")
        .arg(&iface)
        .output()
        .map_err(|e| format!("ifconfig: {}", e))?;
    let ifconfig_str = String::from_utf8_lossy(&ifconfig.stdout).to_string();

    let mut local_ip = String::new();
    let mut subnet_mask = String::new();
    for line in ifconfig_str.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("inet ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if !parts.is_empty() && parts[0] != "127.0.0.1" {
                local_ip = parts[0].to_string();
                let nm_idx = parts.iter().position(|&p| p == "netmask");
                if let Some(i) = nm_idx {
                    if i + 1 < parts.len() {
                        let raw = parts[i + 1];
                        if let Some(hex) = raw.strip_prefix("0x") {
                            if let Ok(n) = u32::from_str_radix(hex, 16) {
                                subnet_mask = format!(
                                    "{}.{}.{}.{}",
                                    (n >> 24) & 0xff,
                                    (n >> 16) & 0xff,
                                    (n >> 8) & 0xff,
                                    n & 0xff
                                );
                            }
                        } else {
                            subnet_mask = raw.to_string();
                        }
                    }
                }
            }
        }
    }

    let netstat = Command::new("netstat")
        .args(["-rn", "-f", "inet"])
        .output()
        .map_err(|e| format!("netstat: {}", e))?;
    let netstat_str = String::from_utf8_lossy(&netstat.stdout).to_string();
    let mut gateway = String::new();
    for line in netstat_str.lines() {
        if line.starts_with("default") || line.starts_with("0.0.0.0") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                gateway = parts[1].to_string();
            }
            break;
        }
    }

    let mut dns_servers: Vec<String> = Vec::new();
    if let Ok(dns_out) = Command::new("scutil").args(["--dns"]).output() {
        let dns_str = String::from_utf8_lossy(&dns_out.stdout);
        for line in dns_str.lines() {
            if line.trim().starts_with("nameserver[") {
                if let Some(addr) = line.split(" : ").nth(1) {
                    let addr = addr.trim().to_string();
                    if !dns_servers.contains(&addr) && !addr.is_empty() {
                        dns_servers.push(addr);
                    }
                }
            }
        }
    }

    warm_arp_table(&calc_broadcast(&local_ip, &subnet_mask));
    let arp_out = Command::new("arp").arg("-a").output().map_err(|e| format!("arp: {}", e))?;
    let arp_str = String::from_utf8_lossy(&arp_out.stdout).to_string();
    let mut devices: Vec<DeviceInfo> = Vec::new();
    let mut macs_for_lookup: Vec<String> = Vec::new();
    for line in arp_str.lines() {
        if line.contains("(incomplete)")
            || line.contains("ff:ff:ff:ff:ff:ff")
            || line.contains("224.0.0")
        {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let ip = parts[1].trim_matches(|c| c == '(' || c == ')').to_string();
        let mac = parts[3].to_string();
        if mac.len() < 17 || ip == local_ip || mac.contains("(incomplete)") {
            continue;
        }
        macs_for_lookup.push(mac.clone());
        devices.push(DeviceInfo { ip, mac, vendor: None });
    }

    let vendors = batch_lookup_vendors(&macs_for_lookup);
    for (i, vendor) in vendors.into_iter().enumerate() {
        if i < devices.len() {
            devices[i].vendor = vendor;
        }
    }

    if !gateway.is_empty() && !devices.iter().any(|d| d.ip == gateway) {
        devices.insert(
            0,
            DeviceInfo {
                ip: gateway.clone(),
                mac: String::new(),
                vendor: None,
            },
        );
    }

    Ok(LocalNetworkInfo {
        network_range: calc_network_range(&local_ip, &subnet_mask),
        local_ip,
        gateway,
        subnet_mask,
        dns_servers,
        devices,
    })
}

#[cfg(target_os = "macos")]
fn find_active_iface() -> Result<String, String> {
    use std::process::Command;
    let out = Command::new("ifconfig")
        .output()
        .map_err(|e| format!("ifconfig: {}", e))?;
    let s = String::from_utf8_lossy(&out.stdout);
    for line in s.lines() {
        if line.contains("inet ") && !line.contains("127.0.0.1") {
            if let Some(iface) = find_iface_in_section(&s, line) {
                return Ok(iface);
            }
        }
    }
    Ok("en0".into())
}

#[cfg(target_os = "macos")]
fn find_iface_in_section(text: &str, target_line: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    for i in (0..lines.len()).rev() {
        if lines[i] == target_line {
            for j in (0..i).rev() {
                let trimmed = lines[j].trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(colon) = trimmed.find(':') {
                    let iface = &trimmed[..colon];
                    if !iface.is_empty() && iface.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                        return Some(iface.to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn get_local_network() -> Result<LocalNetworkInfo, String> {
    use std::process::Command;

    let ipconfig = Command::new("ipconfig")
        .output()
        .map_err(|e| format!("ipconfig: {}", e))?;
    let s = String::from_utf8_lossy(&ipconfig.stdout).to_string();
    let mut local_ip = String::new();
    let mut subnet_mask = String::new();
    let mut gateway = String::new();
    let mut dns_servers: Vec<String> = Vec::new();
    let mut capture_dns = false;

    for line in s.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("IPv4 Address") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 2 {
                local_ip = parts.last().unwrap_or(&"").trim().to_string();
            }
        }
        if let Some(rest) = trimmed.strip_prefix("Subnet Mask") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 2 {
                subnet_mask = parts.last().unwrap_or(&"").trim().to_string();
            }
        }
        if let Some(rest) = trimmed.strip_prefix("Default Gateway") {
            let parts: Vec<&str> = rest.split(':').collect();
            if parts.len() >= 2 && gateway.is_empty() {
                gateway = parts.last().unwrap_or(&"").trim().to_string();
            }
        }
        if trimmed.starts_with("DNS Servers") {
            capture_dns = true;
            continue;
        }
        if capture_dns && trimmed.is_empty() {
            capture_dns = false;
        }
        if capture_dns && !trimmed.is_empty() && !trimmed.ends_with(':') {
            let addr = trimmed.to_string();
            if !dns_servers.contains(&addr) {
                dns_servers.push(addr);
            }
        }
    }

    warm_arp_table(&calc_broadcast(&local_ip, &subnet_mask));
    let arp_out = Command::new("arp")
        .arg("-a")
        .output()
        .map_err(|e| format!("arp: {}", e))?;
    let arp_str = String::from_utf8_lossy(&arp_out.stdout).to_string();
    let mut devices: Vec<DeviceInfo> = Vec::new();
    let mut macs_for_lookup: Vec<String> = Vec::new();
    for line in arp_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let ip = parts[0].to_string();
        let mac = parts[1].to_string();
        if mac == "---" || ip == local_ip || mac.len() < 17 {
            continue;
        }
        macs_for_lookup.push(mac.clone());
        devices.push(DeviceInfo { ip, mac, vendor: None });
    }

    let vendors = batch_lookup_vendors(&macs_for_lookup);
    for (i, vendor) in vendors.into_iter().enumerate() {
        if i < devices.len() {
            devices[i].vendor = vendor;
        }
    }

    if !gateway.is_empty() && !devices.iter().any(|d| d.ip == gateway) {
        devices.insert(
            0,
            DeviceInfo {
                ip: gateway.clone(),
                mac: String::new(),
                vendor: None,
            },
        );
    }

    Ok(LocalNetworkInfo {
        network_range: calc_network_range(&local_ip, &subnet_mask),
        local_ip,
        gateway,
        subnet_mask,
        dns_servers,
        devices,
    })
}

#[cfg(target_os = "linux")]
fn get_local_network() -> Result<LocalNetworkInfo, String> {
    use std::process::Command;

    let ip_out = Command::new("ip")
        .args(["-4", "addr", "show"])
        .output()
        .map_err(|e| format!("ip addr: {}", e))?;
    let s = String::from_utf8_lossy(&ip_out.stdout).to_string();
    let mut local_ip = String::new();
    for line in s.lines() {
        if let Some(rest) = line.trim().strip_prefix("inet ") {
            let addr = rest.split_whitespace().next().unwrap_or("");
            if addr != "127.0.0.1" && !addr.is_empty() {
                local_ip = addr.to_string();
                break;
            }
        }
    }

    let route = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .map_err(|e| format!("ip route: {}", e))?;
    let rs = String::from_utf8_lossy(&route.stdout).to_string();
    let mut gateway = String::new();
    for line in rs.lines() {
        if let Some(rest) = line.strip_prefix("default via ") {
            gateway = rest.split_whitespace().next().unwrap_or("").to_string();
            break;
        }
    }

    let mut subnet_mask = String::new();
    for line in s.lines() {
        if line.contains(&local_ip) {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            for i in 0..parts.len() {
                if parts[i] == &local_ip && i + 1 < parts.len() {
                    let cidr = parts[i + 1];
                    if let Some(c) = cidr.strip_prefix('/') {
                        if let Ok(prefix) = c.parse::<u8>() {
                            subnet_mask = cidr_to_mask(prefix);
                        }
                    }
                }
            }
        }
    }

    let mut dns_servers: Vec<String> = Vec::new();
    if let Ok(resolv) = std::fs::read_to_string("/etc/resolv.conf") {
        for line in resolv.lines() {
            if let Some(rest) = line.trim().strip_prefix("nameserver ") {
                let addr = rest.trim().to_string();
                if !dns_servers.contains(&addr) && !addr.is_empty() {
                    dns_servers.push(addr);
                }
            }
        }
    }
    if dns_servers.is_empty() {
        if let Ok(dns_out) = Command::new("resolvectl").args(["dns"]).output() {
            let ds = String::from_utf8_lossy(&dns_out.stdout);
            for line in ds.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for p in parts {
                    if p.contains('.') && !p.contains('/') {
                        let addr = p.to_string();
                        if !dns_servers.contains(&addr) {
                            dns_servers.push(addr);
                        }
                    }
                }
            }
        }
    }

    warm_arp_table(&calc_broadcast(&local_ip, &subnet_mask));
    let arp_out = Command::new("arp")
        .arg("-n")
        .output()
        .map_err(|e| format!("arp: {}", e))?;
    let arp_str = String::from_utf8_lossy(&arp_out.stdout).to_string();
    let mut devices: Vec<DeviceInfo> = Vec::new();
    let mut macs_for_lookup: Vec<String> = Vec::new();
    for line in arp_str.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let ip = parts[0].to_string();
        let mac = parts[2].to_string();
        if mac == "(incomplete)" || ip == local_ip || mac.len() < 17 {
            continue;
        }
        macs_for_lookup.push(mac.clone());
        devices.push(DeviceInfo { ip, mac, vendor: None });
    }

    let vendors = batch_lookup_vendors(&macs_for_lookup);
    for (i, vendor) in vendors.into_iter().enumerate() {
        if i < devices.len() {
            devices[i].vendor = vendor;
        }
    }

    if !gateway.is_empty() && !devices.iter().any(|d| d.ip == gateway) {
        devices.insert(
            0,
            DeviceInfo {
                ip: gateway.clone(),
                mac: String::new(),
                vendor: None,
            },
        );
    }

    Ok(LocalNetworkInfo {
        network_range: calc_network_range(&local_ip, &subnet_mask),
        local_ip,
        gateway,
        subnet_mask,
        dns_servers,
        devices,
    })
}

#[cfg(target_os = "linux")]
fn cidr_to_mask(prefix: u8) -> String {
    if prefix == 0 {
        return "0.0.0.0".into();
    }
    if prefix >= 32 {
        return "255.255.255.255".into();
    }
    let mask: u32 = !0u32 << (32 - prefix);
    format!(
        "{}.{}.{}.{}",
        (mask >> 24) & 0xff,
        (mask >> 16) & 0xff,
        (mask >> 8) & 0xff,
        mask & 0xff
    )
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn get_local_network() -> Result<LocalNetworkInfo, String> {
    Ok(LocalNetworkInfo {
        local_ip: String::new(),
        gateway: String::new(),
        subnet_mask: String::new(),
        network_range: String::new(),
        dns_servers: vec![],
        devices: vec![],
    })
}

#[tauri::command]
pub async fn get_local_network_info() -> Result<LocalNetworkInfo, String> {
    tauri::async_runtime::spawn_blocking(move || get_local_network())
        .await
        .map_err(|e| e.to_string())?
}
