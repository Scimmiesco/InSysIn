use crate::models::hardware::{CoreInfo, DiskUsage, GpuInfo, MemInfo, NetworkUsage, ProcessInfo, SystemInfo, SysStats};
use crate::services::memory_service;
use std::collections::HashMap;
use std::sync::OnceLock;
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

const PROCESS_LIMIT: usize = 15;

static GPU_CACHE: OnceLock<Option<GpuInfo>> = OnceLock::new();

#[cfg(target_os = "macos")]
fn sysctl_u32(name: &str) -> Option<u32> {
    use std::ffi::CString;
    let cname = CString::new(name).ok()?;
    let mut val: libc::c_uint = 0;
    let mut len = std::mem::size_of::<libc::c_uint>();
    let ret = unsafe {
        libc::sysctlbyname(
            cname.as_ptr(),
            &mut val as *mut _ as *mut libc::c_void,
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 { Some(val) } else { None }
}

#[cfg(not(target_os = "macos"))]
fn sysctl_u32(_name: &str) -> Option<u32> {
    None
}

fn detect_core_layout(sys: &System) -> (u32, u32) {
    let p_cores = sysctl_u32("hw.perflevel0.physicalcpu");
    let e_cores = sysctl_u32("hw.perflevel1.physicalcpu");
    if let (Some(p), Some(e)) = (p_cores, e_cores) {
        return (p, e);
    }
    let p: u32 = sys.cpus().iter().filter(|c| {
        let n = c.name().to_lowercase();
        n.contains("pmp")
    }).count() as u32;
    let e: u32 = sys.cpus().iter().filter(|c| {
        let n = c.name().to_lowercase();
        n.contains("emp")
    }).count() as u32;
    if p > 0 || e > 0 {
        return (p, e);
    }
    (sys.cpus().len() as u32, 0)
}

fn physical_core_count() -> (u32, u32) {
    let physical = sysctl_u32("hw.physicalcpu");
    let logical = sysctl_u32("hw.logicalcpu");
    if let (Some(p), Some(l)) = (physical, logical) {
        return (p, l);
    }
    let logical = sysinfo::System::new_all().cpus().len() as u32;
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/sys/devices/system/cpu/present") {
            if let Some(max) = s.trim().split('-').last().and_then(|n| n.parse::<u32>().ok()) {
                return (logical, max + 1);
            }
        }
    }
    (logical, logical)
}

fn collect_cpu_temperature() -> Option<f32> {
    #[cfg(target_os = "macos")]
    {
        let temp_raw = sysctl_u32("dev.cpu.0.temperature");
        if let Some(t) = temp_raw {
            let celsius = t as f32 / 10.0;
            if celsius > 0.0 && celsius < 150.0 {
                return Some(celsius);
            }
        }
        None
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
            let mut temps: Vec<f32> = Vec::new();
            for entry in entries.flatten() {
                let zone_dir = entry.path();
                let type_path = zone_dir.join("type");
                if let Ok(kind) = std::fs::read_to_string(&type_path) {
                    let kind = kind.trim().to_lowercase();
                    if kind.contains("cpu") || kind.contains("x86_pkg") || kind.contains("acpitz") {
                        let temp_path = zone_dir.join("temp");
                        if let Ok(content) = std::fs::read_to_string(&temp_path) {
                            if let Ok(mc) = content.trim().parse::<f32>() {
                                let celsius = mc / 1000.0;
                                if celsius > 0.0 && celsius < 150.0 {
                                    temps.push(celsius);
                                }
                            }
                        }
                    }
                }
            }
            if !temps.is_empty() {
                let avg = temps.iter().sum::<f32>() / temps.len() as f32;
                return Some(avg);
            }
        }
        None
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn collect_gpu_temperature() -> Option<f32> {
    #[cfg(target_os = "macos")]
    {
        sysctl_u32("dev.cpu.0.gpu_temperature")
            .map(|t| t as f32 / 10.0)
            .filter(|&c| c > 0.0 && c < 150.0)
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn collect_user_system_cpu() -> (f32, f32) {
    use std::sync::Mutex;
    static PREV: Mutex<Option<(f32, f32, f32)>> = Mutex::new(None);

    const CPU_STATE_MAX: usize = 4;
    const CPU_STATE_USER: usize = 0;
    const CPU_STATE_SYSTEM: usize = 1;
    const CPU_STATE_IDLE: usize = 2;
    const CPU_STATE_NICE: usize = 3;

    unsafe {
        let mut processor_count: libc::natural_t = 0;
        let mut processor_info: libc::processor_info_array_t = std::ptr::null_mut();
        let mut info_count: libc::mach_msg_type_number_t = 0;

        let ret = libc::host_processor_info(
            libc::mach_host_self(),
            libc::PROCESSOR_CPU_LOAD_INFO,
            &mut processor_count,
            &mut processor_info,
            &mut info_count,
        );

        if ret != 0 || processor_info.is_null() {
            return (0.0, 0.0);
        }

        let info_slice = std::slice::from_raw_parts(processor_info as *const i32, processor_count as usize * CPU_STATE_MAX);

        let mut user_ticks: f32 = 0.0;
        let mut system_ticks: f32 = 0.0;
        let mut idle_ticks: f32 = 0.0;
        let mut nice_ticks: f32 = 0.0;

        for i in 0..processor_count as usize {
            let base = i * CPU_STATE_MAX;
            user_ticks += info_slice[base + CPU_STATE_USER] as f32;
            system_ticks += info_slice[base + CPU_STATE_SYSTEM] as f32;
            idle_ticks += info_slice[base + CPU_STATE_IDLE] as f32;
            nice_ticks += info_slice[base + CPU_STATE_NICE] as f32;
        }

        libc::vm_deallocate(
            libc::mach_task_self(),
            processor_info as libc::vm_address_t,
            info_count as libc::vm_size_t,
        );

        let total = user_ticks + system_ticks + idle_ticks + nice_ticks;
        if total <= 0.0 {
            return (0.0, 0.0);
        }

        let (user_pct, system_pct) = {
            let mut prev = PREV.lock().unwrap();
            if let Some((pu, ps, pt)) = *prev {
                let du = user_ticks - pu;
                let ds = system_ticks - ps;
                let dt = total - pt;
                *prev = Some((user_ticks, system_ticks, total));
                if dt > 0.0 {
                    ((du / dt * 100.0).clamp(0.0, 100.0), (ds / dt * 100.0).clamp(0.0, 100.0))
                } else {
                    (0.0, 0.0)
                }
            } else {
                *prev = Some((user_ticks, system_ticks, total));
                (0.0, 0.0)
            }
        };

        (user_pct, system_pct)
    }
}

#[cfg(target_os = "linux")]
fn collect_user_system_cpu() -> (f32, f32) {
    use std::sync::Mutex;
    static PREV: Mutex<Option<[f32; 3]>> = Mutex::new(None);

    if let Ok(content) = std::fs::read_to_string("/proc/stat") {
        for line in content.lines() {
            if line.starts_with("cpu ") {
                let parts: Vec<f32> = line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() >= 4 {
                    let user = parts.get(0).copied().unwrap_or(0.0);
                    let nice = parts.get(1).copied().unwrap_or(0.0);
                    let system = parts.get(2).copied().unwrap_or(0.0);
                    let idle = parts.get(3).copied().unwrap_or(0.0);
                    let user_total = user + nice;
                    let total = user_total + system + idle;

                    if total <= 0.0 {
                        return (0.0, 0.0);
                    }

                    let mut prev = PREV.lock().unwrap();
                    if let Some(p) = *prev {
                        let du = user_total - p[0];
                        let ds = system - p[1];
                        let dt = total - p[2];
                        *prev = Some([user_total, system, total]);
                        if dt > 0.0 {
                            return ((du / dt * 100.0).clamp(0.0, 100.0), (ds / dt * 100.0).clamp(0.0, 100.0));
                        }
                    } else {
                        *prev = Some([user_total, system, total]);
                    }
                }
                break;
            }
        }
    }
    (0.0, 0.0)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn collect_user_system_cpu() -> (f32, f32) {
    (0.0, 0.0)
}

#[cfg(target_os = "macos")]
fn collect_per_core_user_system() -> Vec<(f32, f32)> {
    const CPU_STATE_MAX: usize = 4;
    const CPU_STATE_USER: usize = 0;
    const CPU_STATE_SYSTEM: usize = 1;

    use std::sync::Mutex;
    static PREV_CORES: Mutex<Option<Vec<(f32, f32, f32)>>> = Mutex::new(None);

    unsafe {
        let mut processor_count: libc::natural_t = 0;
        let mut processor_info: libc::processor_info_array_t = std::ptr::null_mut();
        let mut info_count: libc::mach_msg_type_number_t = 0;

        let ret = libc::host_processor_info(
            libc::mach_host_self(),
            libc::PROCESSOR_CPU_LOAD_INFO,
            &mut processor_count,
            &mut processor_info,
            &mut info_count,
        );

        if ret != 0 || processor_info.is_null() {
            return vec![(0.0, 0.0); processor_count as usize];
        }

        let info_slice = std::slice::from_raw_parts(processor_info as *const i32, processor_count as usize * CPU_STATE_MAX);

        let mut per_core: Vec<(f32, f32, f32)> = Vec::with_capacity(processor_count as usize);
        for i in 0..processor_count as usize {
            let base = i * CPU_STATE_MAX;
            let user = info_slice[base + CPU_STATE_USER] as f32 + info_slice[base + 3] as f32; // user + nice
            let system = info_slice[base + CPU_STATE_SYSTEM] as f32;
            let idle = info_slice[base + 2] as f32;
            per_core.push((user, system, idle));
        }

        libc::vm_deallocate(
            libc::mach_task_self(),
            processor_info as libc::vm_address_t,
            info_count as libc::vm_size_t,
        );

        let mut results = vec![(0.0f32, 0.0f32); processor_count as usize];
        {
            let mut prev = PREV_CORES.lock().unwrap();
            if let Some(ref prev_data) = *prev {
                for (j, (curr, prev)) in per_core.iter().zip(prev_data.iter()).enumerate() {
                    let &(user, system, idle) = curr;
                    let &(pu, ps, pt) = prev;
                    let total = user + system + idle;
                    let dt = total - pt;
                    if dt > 0.0 {
                        results[j] = (
                            ((user - pu) / dt * 100.0).clamp(0.0, 100.0),
                            ((system - ps) / dt * 100.0).clamp(0.0, 100.0),
                        );
                    }
                }
            }
            *prev = Some(per_core.iter().map(|&(u, s, i)| (u, s, u + s + i)).collect());
        }

        results
    }
}

#[cfg(not(target_os = "macos"))]
fn collect_per_core_user_system() -> Vec<(f32, f32)> {
    Vec::new()
}

fn collect_gpu_info() -> Option<GpuInfo> {
    GPU_CACHE.get_or_init(|| {
        #[cfg(target_os = "macos")]
        {
            return detect_macos_gpu();
        }
        #[cfg(not(target_os = "macos"))]
        None
    }).clone()
}

#[cfg(target_os = "macos")]
fn detect_macos_gpu() -> Option<GpuInfo> {
    let output = std::process::Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&s).ok()?;
    let displays = json.get("SPDisplaysDataType")?.as_array()?;
    for gpu in displays {
        let bus = gpu.get("sppci_bus").and_then(|v| v.as_str()).unwrap_or("");
        let model = gpu.get("sppci_model").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let vendor = gpu.get("spdisplays_vendor").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let cores = gpu.get("sppci_cores").and_then(|v| v.as_str()).unwrap_or("0").parse().unwrap_or(0);
        let integrated = bus == "spdisplays_builtin";
        if !model.is_empty() {
            return Some(GpuInfo { model, vendor, integrated, cores, gpu_temperature: collect_gpu_temperature() });
        }
    }
    None
}

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
        let mut total = 0u64;
        let mut available = 0u64;
        for disk in disks.list() {
            read += disk.usage().read_bytes;
            write += disk.usage().written_bytes;
            total += disk.total_space();
            available += disk.available_space();
        }
        DiskUsage {
            read_bytes: read,
            write_bytes: write,
            total_bytes: total,
            available_bytes: available,
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
    let (physical_cores, logical_cores) = physical_core_count();
    let hyperthreading = logical_cores > physical_cores;
    let (p_cores, e_cores) = detect_core_layout(sys);

    let per_core_us = collect_per_core_user_system();

    let cores: Vec<CoreInfo> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let kind = if i < p_cores as usize {
                "performance"
            } else if i < (p_cores + e_cores) as usize {
                "efficiency"
            } else {
                "unknown"
            };
            let physical = i < physical_cores as usize;
            let (core_user, core_system) = per_core_us.get(i).copied().unwrap_or((0.0, 0.0));
            CoreInfo {
                usage: c.cpu_usage(),
                name: c.name().to_string(),
                kind: kind.to_string(),
                frequency: c.frequency(),
                physical,
                user: core_user,
                system: core_system,
            }
        })
        .collect();

    let gpu = collect_gpu_info();
    let (cpu_user, cpu_system) = collect_user_system_cpu();

    SysStats {
        system_info: SystemInfo {
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            uptime_secs: System::uptime(),
            cpu_brand,
            cpu_cores,
            physical_cores,
            performance_cores: p_cores,
            efficiency_cores: e_cores,
            hyperthreading,
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
        cpu_temperature: collect_cpu_temperature(),
        cpu_user,
        cpu_system,
        cores,
        processes,
        disk_usage,
        network_usage,
        gpu,
    }
}
