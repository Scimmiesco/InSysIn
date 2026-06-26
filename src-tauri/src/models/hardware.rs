use serde::Serialize;

#[derive(Serialize)]
pub struct MemInfo {
    pub total_memory: u64,
    pub free_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,
    pub used_swap: u64,
    pub total_swap: u64,
    pub free_swap: u64,
}

// 1. Nova Struct para os Processos
#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64, // em bytes
}

#[derive(Serialize)]
pub struct SysStats {
    pub mem_info: MemInfo,
    pub cpu_usage: f32,
    pub processes: Vec<ProcessInfo>, // 2. Adicionando a lista de processos aqui
}
