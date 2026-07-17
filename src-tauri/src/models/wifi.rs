use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct InternetInfo {
    pub public_ip: String,
    pub isp: String,
    pub city: String,
    pub country: String,
    pub org: String,
    pub timezone: String,
    pub asn: String,
    pub latency_ms: f64,
    pub ping_target: String,
    pub online: bool,
}

#[derive(Serialize, Clone)]
pub struct SpeedResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
}

#[derive(Serialize, Clone)]
pub struct InternetDiagnostics {
    pub info: InternetInfo,
    pub speed: Option<SpeedResult>,
}
