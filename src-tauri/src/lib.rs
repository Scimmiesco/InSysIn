use serde::Serialize;
use std::sync::Mutex;
use sysinfo::System;
use tauri::State; // Necessário importar o State para o Tauri gerenciar a memória

// #[derive(Serialize)] é uma "Macro". Ela escreve código automaticamente por baixo dos panos
// para converter essa struct em um JSON que o seu Angular consiga ler.
#[derive(Serialize)]
struct SysStats {
    // "u64" significa Unsigned Integer (Inteiro sem sinal negativo) de 64 bits.
    total_memory: u64,
    used_memory: u64,
    // "f32" significa Float (número decimal) de 32 bits.
    cpu_usage: f32,
}

// Criamos uma struct para manter o sistema vivo na memória durante toda a execução.
struct AppState {
    // O Rust é paranóico com segurança (Data Races). Se duas partes tentarem acessar a
    // memória ao mesmo tempo, ele nem compila. O "Mutex" cria uma "fila" (trava):
    // garante que apenas uma coisa possa ler o System por vez.
    sys: Mutex<System>,
}

// A macro #[tauri::command] avisa o compilador que o Frontend vai chamar essa função.
// O parâmetro `state` recebe o AppState injetado automaticamente pelo Tauri.
// O `<'_>` lida com "Lifetimes", dizendo ao Rust que essa referência de estado é válida agora.
#[tauri::command]
fn ler_hardware(state: State<'_, AppState>) -> SysStats {
    // 1. state.sys.lock(): Pede a chave do Mutex para acessar o System.
    // 2. unwrap(): O Rust não usa "null". Funções que podem falhar retornam um "Result".
    // O "unwrap" é uma forma de dizer: "Confia, pode abrir o resultado, não vai dar erro".
    // 3. let mut: Em Rust, variáveis são IMUTÁVEIS por padrão. Usamos 'mut' para poder alterá-la.
    let mut sys = state.sys.lock().unwrap();

    // Agora ele atualiza os dados baseados no estado salvo, permitindo calcular o % de CPU.
    sys.refresh_all();

    // Em Rust, a última expressão de uma função NÃO precisa da palavra "return".
    // Basta omitir o ponto-e-vírgula (;) no final da chave "}".
    SysStats {
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        cpu_usage: sys.global_cpu_info().cpu_usage(),
    }
}

// #[cfg_attr] são instruções condicionais de compilação.
// Isso só será compilado se o alvo for Mobile.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut sys = System::new_all();
    sys.refresh_all(); // Coleta os dados iniciais.

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // .manage() pega a sua struct e diz ao Tauri: "Guarde isso na memória até o app fechar".
        .manage(AppState {
            sys: Mutex::new(sys),
        })
        .invoke_handler(tauri::generate_handler![ler_hardware])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
