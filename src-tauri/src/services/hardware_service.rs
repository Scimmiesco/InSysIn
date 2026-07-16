use crate::models::hardware::{DiskUsage, MemInfo, NetworkUsage, ProcessInfo, SystemInfo, SysStats};
use crate::services::memory_service;
use std::collections::HashMap;
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

const PROCESS_LIMIT: usize = 15;

pub fn coletar_dados(sys: &mut System, disks: &mut Disks, networks: &mut Networks) -> SysStats {
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    disks.refresh(true);
    networks.refresh(true);

    let cpu_count = sys.cpus().len() as f32;
    let mut grouped: HashMap<String, ProcessInfo> = HashMap::new();

    for (pid, process) in sys.processes() {
        let name = process
            .exe()
            .and_then(|path| path.file_name())
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_else(|| process.name().to_string_lossy().to_string());

        let entry = grouped.entry(name.clone()).or_insert(ProcessInfo {
            pid: pid.as_u32(),
            name,
            cpu_usage: 0.0,
            memory_usage: 0,
        });
        entry.cpu_usage += process.cpu_usage() / cpu_count;
        entry.memory_usage = entry.memory_usage.max(process.memory());
    }

    let mut processes: Vec<ProcessInfo> = grouped.into_values().collect();

    if processes.len() > PROCESS_LIMIT {
        processes.select_nth_unstable_by(PROCESS_LIMIT - 1, |a, b| {
            b.memory_usage
                .partial_cmp(&a.memory_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(PROCESS_LIMIT);
    }

    processes.sort_unstable_by(|a, b| {
        b.memory_usage
            .partial_cmp(&a.memory_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let disk_usage = {
        let mut read = 0u64;
        let mut write = 0u64;
        for disk in disks.list() {
            read += disk.usage().read_bytes;
            write += disk.usage().written_bytes;
        }
        DiskUsage {
            read_bytes: read,
            write_bytes: write,
        }
    };

    let network_usage = {
        let mut received = 0u64;
        let mut transmitted = 0u64;
        for (_, data) in &*networks {
            received += data.received();
            transmitted += data.transmitted();
        }
        NetworkUsage {
            received_bytes: received,
            transmitted_bytes: transmitted,
        }
    };

    let breakdown = memory_service::coletar_breakdown();

    let cpu_brand = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
    let cpu_cores = sys.cpus().len() as u32;

    SysStats {
        system_info: SystemInfo {
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            uptime_secs: System::uptime(),
            cpu_brand,
            cpu_cores,
            total_processes: sys.processes().len(),
        },
        mem_info: MemInfo {
            total_memory: sys.total_memory(),
            free_memory: sys.free_memory(),
            available_memory: sys.available_memory(),
            used_memory: sys.used_memory(),
            used_swap: sys.used_swap(),
            total_swap: sys.total_swap(),
            free_swap: sys.free_swap(),
            breakdown,
        },
        cpu_usage: sys.global_cpu_usage(),
        processes,
        disk_usage,
        network_usage,
    }
}
