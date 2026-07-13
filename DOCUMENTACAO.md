# InSysIn — Documentação Técnica

> **Monitor de sistema desktop** — Tauri v2 + Angular 22 + Rust 🦀

---

## Estrutura do Projeto

```
InSysIn/
├── src/                          # Frontend Angular 22
│   ├── app/
│   │   ├── app.component.*       # Componente raiz
│   │   ├── app.routes.ts         # Rotas
│   │   └── pages/home/           # Página principal (monitor)
│   ├── generated/                # Tipos e comandos Tauri gerados
│   ├── assets/                   # SVGs (angular, tauri)
│   └── index.html / main.ts      # Entrypoints Angular
│
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── main.rs               # Entrypoint (Windows subsystem)
│   │   ├── lib.rs                # Tauri builder, AppState
│   │   ├── hardware.rs           # Coleta de estatísticas do sistema
│   │   ├── historico.rs          # Leitura do histórico + agrupamento
│   │   ├── models/               # Structs Serialize (SysStats, MemInfo, etc.)
│   │   └── db/                   # SQLite (init, save, read)
│   ├── tauri.conf.json           # Config da janela, build, bundle
│   ├── Cargo.toml                # Dependências Rust
│   └── icons/                    # Ícones do app
│
├── angular.json                  # Config do Angular CLI
├── package.json                  # Dependências Node
├── tsconfig.json                 # TypeScript
└── .gitignore
```

---

## 🖥️ Frontend (Angular 22)

### Stack

| Tecnologia | Versão | Função |
|-----------|--------|--------|
| Angular | 22 | Framework SPA |
| TypeScript | 6.0 | Linguagem |
| @tauri-apps/api | 2 | Ponte com Rust |
| RxJS | 7.8 | Reatividade |
| Angular build | 22 | Vite/esbuild |

### Componentes

#### `src/app/pages/home/home.ts`
- **Signals** reativos (`sys_info`, `sys_history`, `processos_agrupados`, `ordenacaoAtual`)
- **Ciclo de vida**:
  - `ngOnInit`: dispara leituras iniciais
  - `setInterval(1000ms)`: `lerHardware()` → CPU/RAM/processos em tempo real
  - `setInterval(30000ms)`: `obterHistorico()` → atualiza histórico do SQLite
- **Métodos**:
  - `lerDados()` → chama comando Tauri `lerHardware`
  - `obterHistorico()` → chama comando Tauri `obterHistorico`
  - `filtrarPor(coluna)` → ordena processos agrupados (nome/cpu/memoria/data)
  - `get statsHistory` → extrai últimos 20 snapshots para gráfico

#### `src/app/pages/home/home.html`
- **Seção Memory**: RAM e Swap usados / total (GB)
- **Seção CPU**: barra gradiente azul com porcentagem animada
- **Seção System History**: gráfico de barras dos últimos 20 snapshots com tooltip
- **Seção Processes**: tabela top 15 processos por memória (tempo real)
- **Seção Aggregated Process History**: tabela agrupada com ordenação clicável

#### `src/app/pages/home/home.css`
- Tema escuro (`#0d1117` fundo, `#161b22` cards)
- Bordas arredondadas, fonte monospace para números
- Barra de CPU com gradiente `#58a6ff → #1f6feb`
- Scrollbar customizada, hover nas linhas, tooltips no gráfico

---

## 🦀 Backend (Rust + Tauri)

### Stack

| Crate | Versão | Função |
|-------|--------|--------|
| tauri | 2 | Framework desktop |
| sysinfo | 0.39.5 | Leitura de hardware (CPU, RAM, processos) |
| rusqlite | 0.40.1 | SQLite com WAL |
| serde + serde_json | 1 | Serialização JSON |
| chrono | 0.4 | Timestamps |
| tauri-ts-generator | 2.1 | Geração automática de tipos TS |

### Módulos

#### `src-tauri/src/lib.rs` — Entrypoint
- Cria `AppState` global (compartilhado entre comandos)
  - `sys: Mutex<System>` — instância única do sysinfo (performance)
  - `last_db_save: Mutex<Instant>` — controle do intervalo de 3 min
- Registra plugins: `tauri_plugin_opener`
- Registra comandos: `ler_hardware`, `obter_historico`

#### `src-tauri/src/hardware.rs` — Coleta de Dados
- Função `ler_hardware` (comando Tauri):
  1. **Refresh seletivo**: `refresh_cpu_usage()` + `refresh_memory()` + `refresh_processes()`
  2. **Agrupa processos** por nome (soma CPU, maior memória)
  3. **Top 15** por uso de memória (usando `select_nth_unstable_by` para performance)
  4. Monta struct `SysStats` com `MemInfo`, `cpu_usage`, `processes`
  5. **A cada 3 minutos**: salva snapshot no SQLite via `salvar_historico()`

#### `src-tauri/src/historico.rs` — Histórico
- `obter_historico` (comando Tauri): retorna `HistoricoCompleto` com:
  - Últimos 60 registros de `uso_sistema`
  - Últimos 300 registros de `uso_processos`
- `obter_processos_agrupados` (comando Tauri): retorna processos agrupados por nome com:
  - `MAX(data_hora)`, `SUM(cpu)`, `SUM(memoria)`
  - **Proteção anti-SQL injection**: validação estrita da coluna de ordenação

#### `src-tauri/src/main.rs` — Entrypoint final

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() { insysin_lib::run() }
```

### Modelos (`src-tauri/src/models/`)

| Struct | Campos |
|--------|--------|
| `AppState` | `sys: Mutex<System>`, `last_db_save: Mutex<Instant>` |
| `SysStats` | `mem_info: MemInfo`, `cpu_usage: f32`, `processes: Vec<ProcessInfo>` |
| `MemInfo` | `total_memory`, `free_memory`, `available_memory`, `used_memory`, `used_swap`, `total_swap`, `free_swap` |
| `ProcessInfo` | `pid: u32`, `name: String`, `cpu_usage: f32`, `memory_usage: u64` |
| `HistoricoSistema` | `data_hora`, `cpu_global`, `ram_usada` |
| `HistoricoProcesso` | `data_hora`, `nome`, `cpu`, `memoria` |
| `HistoricoCompleto` | `sistema: Vec<HistoricoSistema>`, `processos: Vec<HistoricoProcesso>` |
| `ProcessoAgrupado` | `nome`, `ultima_data`, `total_cpu`, `total_memoria` |

---

## 🗄️ Banco de Dados (SQLite)

### Configuração
- **Arquivo**: `historico_sistema.sqlite` (gerado na raiz do app)
- **Engine**: WAL (Write-Ahead Logging) para performance
- **Synchronous**: NORMAL

### Tabelas

#### `uso_sistema`
| Coluna | Tipo | Descrição |
|--------|------|-----------|
| `id` | INTEGER PK | Auto incremento |
| `data_hora` | TEXT | ISO 8601 |
| `cpu_global` | REAL | Uso global da CPU (%) |
| `ram_usada` | INTEGER | RAM usada em bytes |

#### `uso_processos`
| Coluna | Tipo | Descrição |
|--------|------|-----------|
| `id` | INTEGER PK | Auto incremento |
| `data_hora` | TEXT | ISO 8601 |
| `nome` | TEXT | Nome do processo |
| `cpu` | REAL | Uso de CPU do processo (%) |
| `memoria` | INTEGER | Memória usada em bytes |

### Fluxo de Dados

```
sysinfo (a cada 1s)
    ↓ ler_hardware() retorna dados em tempo real
    ↓ a cada 180s (3 min): salvar_historico()
        ↓ INSERT em uso_sistema + uso_processos (transação)
            ↓ WAL commit assíncrono
                ↓ obter_historico() / obter_processos_agrupados() leem via SELECT
```

---

## 🔗 Integração (Bridge Rust ↔ TypeScript)

### Como funciona

1. **Lado Rust**: funções com `#[tauri::command]` são registradas em `lib.rs`:

   ```rust
   .invoke_handler(tauri::generate_handler![
       hardware::ler_hardware,
       historico::obter_historico,
       historico::obter_processos_agrupados
   ])
   ```

2. **Gerador automático**: `tauri-ts-generator` (v2.1.0) escaneia os comandos Rust e gera:
   - `src/generated/commands.ts` — funções `invoke()` tipadas
   - `src/generated/types.ts` — interfaces TypeScript espelhando as structs Rust

3. **Lado Angular**: importa e chama como função assíncrona:

   ```typescript
   import { lerHardware, obterHistorico } from "../../../generated/commands";

   const stats = await lerHardware();   // → SysStats
   const hist = await obterHistorico(); // → HistoricoCompleto
   ```

### Config do Tauri (`tauri.conf.json`)

```json
{
  "build": {
    "beforeDevCommand": "npm run start",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist/insysin/browser"
  },
  "app": {
    "windows": [{ "title": "insysin", "width": 800, "height": 600 }]
  }
}
```

---

## 🚀 Build & Deploy

### Requisitos para Build

| Ferramenta | Versão | Como instalar |
|-----------|--------|--------------|
| Rust | nightly/stable | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` |
| Node.js | 20+ | nvm ou site oficial |
| Tauri CLI | 2 | `cargo install tauri-cli` ou `npm install @tauri-apps/cli` |
| Dependências Linux | — | `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev` |

### Build de Produção

**Pipeline:**
1. `beforeBuildCommand`: `npm run build` → Angular compila para `dist/insysin/browser`
2. `cargo build --release` → Rust compila com otimizações:
   - `codegen-units = 1`
   - `lto = true`
   - `opt-level = 3`
   - `panic = "abort"`
   - `strip = true`
3. Tauri empacota tudo em um instalador nativo

### Artefatos Gerados

```
src-tauri/target/release/
├── insysin              # Binário Linux (dev)
└── bundle/
    ├── deb/             # .deb (Debian/Ubuntu)
    ├── rpm/             # .rpm (Fedora)
    ├── AppImage/        # AppImage (universal Linux)
    └── msi/             # .msi (Windows)
```

### Deploy (Distribuição)
- **Linux**: `.deb` / `.rpm` / `.AppImage`
- **Windows**: `.msi` ou `.exe` (NSIS) — build cruzado via `tauri build --target x86_64-pc-windows-msvc`
- **macOS**: `.dmg` — build no próprio macOS

---

## Diagrama do Fluxo de Dados

```
┌──────────────────────────────────────────────────────────────┐
│  FRONTEND (Angular 22)                                       │
│                                                              │
│  home.ts (1s timer)           home.html (tabelas + gráfico) │
│       │ invoke('lerHardware')        ▲                       │
│       ▼                               │                       │
│  lerHardware() ──────── JSON ────────┘                       │
│       │                              │                        │
│       │ invoke('obterHistorico')     │ (30s timer)            │
│       ▼                              │                        │
│  obterHistorico() ────── JSON ───────┘                       │
└──────────────────────────────────────┬───────────────────────┘
                                       │ Tauri IPC
┌──────────────────────────────────────▼───────────────────────┐
│  BACKEND (Rust + Tauri)                                      │
│                                                              │
│  ler_hardware() ──→ sysinfo (CPU, RAM, processos)           │
│       │                                                     │
│       └── a cada 3 min ──→ salvar_historico()              │
│                               │                              │
│  obter_historico() ──→ SQLite (SELECT)                      │
│                               │                              │
│  obter_processos_agrupados() ──→ SQLite (GROUP BY)          │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
                    ┌─────────────────┐
                    │  historico_      │
                    │  sistema.sqlite  │
                    │  (WAL mode)      │
                    └─────────────────┘
```
