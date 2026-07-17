use crate::models::wifi::{InternetDiagnostics, InternetInfo, SpeedResult};
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

#[tauri::command]
pub fn get_internet_info() -> Result<InternetDiagnostics, String> {
    let mut online = false;
    let mut public_ip = String::new();
    let mut isp = String::new();
    let mut city = String::new();
    let mut country = String::new();
    let mut org = String::new();
    let mut timezone = String::new();
    let mut asn_str = String::new();
    let mut latency_ms = 0.0f64;

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
        },
        speed: None,
    })
}

#[tauri::command]
pub fn run_speed_test() -> Result<SpeedResult, String> {
    let down = download_speed()?;
    let up = upload_speed()?;
    Ok(SpeedResult {
        download_mbps: (down * 10.0).round() / 10.0,
        upload_mbps: (up * 10.0).round() / 10.0,
    })
}
