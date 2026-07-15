use serde::Serialize;

#[derive(Serialize)]
pub struct HistoricoSistema {
    pub data_hora: String,
    pub cpu_global: f32,
    pub ram_usada: u64,
    pub ram_total: u64,
}

#[derive(Serialize)]
pub struct HistoricoProcesso {
    pub data_hora: String,
    pub nome: String,
    pub cpu: f32,
    pub memoria: u64,
}

#[derive(Serialize)]
pub struct HistoricoCompleto {
    pub sistema: Vec<HistoricoSistema>,
    pub processos: Vec<HistoricoProcesso>,
}

#[derive(Serialize)]
pub struct ProcessoAgrupado {
    pub nome: String,
    pub ultima_data: String,
    pub total_cpu: f32,
    pub total_memoria: u64,
}
