use crate::models::hardware::{MemInfo, ProcessInfo, SysStats};
use std::collections::HashMap;
use sysinfo::ProcessesToUpdate;
use sysinfo::System;

const PROCESS_LIMIT: usize = 15;

pub fn coletar_dados(sys: &mut System) -> SysStats {
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);

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

    SysStats {
        mem_info: MemInfo {
            total_memory: sys.total_memory(),
            free_memory: sys.free_memory(),
            available_memory: sys.available_memory(),
            used_memory: sys.used_memory(),
            used_swap: sys.used_swap(),
            total_swap: sys.total_swap(),
            free_swap: sys.free_swap(),
        },
        cpu_usage: sys.global_cpu_usage(),
        processes,
    }
}
