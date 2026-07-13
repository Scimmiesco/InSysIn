import { DecimalPipe, DatePipe } from "@angular/common";
import { Component, OnInit, signal } from "@angular/core";
import {
  lerHardware,
  obterHistorico,
  obterProcessosAgrupados,
} from "../../../generated/commands";
import {
  HistoricoCompleto,
  SysStats,
  ProcessoAgrupado,
} from "../../../generated/types";

@Component({
  selector: "app-home",
  imports: [DecimalPipe, DatePipe],
  templateUrl: "./home.html",
  styleUrl: "./home.css",
})
export class Home implements OnInit {
  sys_info = signal<SysStats | null>(null);
  sys_history = signal<HistoricoCompleto | null>(null);
  processos_agrupados = signal<ProcessoAgrupado[]>([]);

  ordenacaoAtual = signal<{ coluna: string; desc: boolean }>({
    coluna: "memoria",
    desc: true,
  });

  ngOnInit(): void {
    this.lerDados();
    this.obterHistorico();
    this.filtrarPor("memoria");

    setInterval(() => {
      this.lerDados();
    }, 1000);

    // Atualiza o histórico a cada 30 segundos
    setInterval(() => {
      this.obterHistorico();
    }, 30000);
  }

  async obterHistorico() {
    try {
      const res = await obterHistorico();
      this.sys_history.set(res);
    } catch (erro) {
      console.error("Erro ao obter histórico:", erro);
    }
  }

  async lerDados() {
    try {
      const res = await lerHardware();
      this.sys_info.set(res);
    } catch (erro) {
      console.error("Erro ao ler hardware:", erro);
    }
  }

  async filtrarPor(coluna: string) {
    try {
      const atual = this.ordenacaoAtual();
      const desc = atual.coluna === coluna ? !atual.desc : true;
      this.ordenacaoAtual.set({ coluna, desc });
      const dados = await obterProcessosAgrupados(coluna, desc);
      this.processos_agrupados.set(dados);
    } catch (erro) {
      console.error("Erro ao buscar dados agrupados:", erro);
    }
  }

  /** Retorna os registros de histórico do sistema (CPU + RAM) */
  get statsHistory(): { cpu: number; ram: number; hora: string }[] {
    const h = this.sys_history();
    if (!h?.uso_sistema) return [];
    return h.uso_sistema.slice(-20).map((s: any) => ({
      cpu: s.cpu_usage ?? 0,
      ram: ((s.used_memory ?? 0) / 1_073_741_824),
      hora: new Date(s.data_hora ?? Date.now()).toLocaleTimeString("pt-BR", { hour: "2-digit", minute: "2-digit" }),
    }));
  }
}
