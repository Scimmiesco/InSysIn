use serde::Serialize;

#[derive(Serialize)]
pub struct MemBreakdown {
    pub app_memory: u64,
    pub wired_memory: u64,
    pub compressed_memory: u64,
    pub cached_memory: u64,
}

#[derive(Serialize)]
pub struct MemInfo {
    pub total_memory: u64,
    pub free_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,
    pub used_swap: u64,
    pub total_swap: u64,
    pub free_swap: u64,
    pub breakdown: Option<MemBreakdown>,
}

#[derive(Serialize)]
pub struct DiskUsage {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

#[derive(Serialize)]
pub struct NetworkUsage {
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime_secs: u64,
    pub cpu_brand: String,
    pub cpu_cores: u32,
    pub total_processes: usize,
}

#[derive(Serialize)]
pub struct SysStats {
    pub system_info: SystemInfo,
    pub mem_info: MemInfo,
    pub cpu_usage: f32,
    pub processes: Vec<ProcessInfo>,
    pub disk_usage: DiskUsage,
    pub network_usage: NetworkUsage,
}
