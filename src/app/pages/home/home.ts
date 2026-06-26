import { DecimalPipe, DatePipe } from "@angular/common";
import { Component, OnInit, signal } from "@angular/core";
import {
  lerHardware,
  obterHistorico,
  obterProcessosAgrupados,
} from "../../../generated/commands";
// Certifique-se de exportar ProcessoAgrupado do seu arquivo de types
import {
  HistoricoCompleto,
  SysStats,
  ProcessoAgrupado,
} from "../../../generated/types";

@Component({
  selector: "app-home",
  imports: [DecimalPipe, DatePipe], // Adicionei o DatePipe aqui
  templateUrl: "./home.html",
  styleUrl: "./home.css",
})
export class Home implements OnInit {
  sys_info = signal<SysStats | null>(null);
  sys_history = signal<HistoricoCompleto | null>(null);

  // NOVO: Signal para a tabela agrupada
  processos_agrupados = signal<ProcessoAgrupado[]>([]);

  // NOVO: Controle de qual coluna está ativa e a direção da seta
  ordenacaoAtual = signal<{ coluna: string; desc: boolean }>({
    coluna: "memoria",
    desc: true,
  });

  ngOnInit(): void {
    this.lerDados();
    this.obterHistorico();

    // Carrega a tabela já ordenada por memória logo que a tela abre
    this.filtrarPor("memoria");

    setInterval(() => {
      this.lerDados();
    }, 1000);
  }

  async obterHistorico() {
    obterHistorico().then((res) => {
      this.sys_history.set(res);
    });
  }

  async lerDados() {
    lerHardware().then((res) => {
      this.sys_info.set(res);
    });
  }

  // NOVO: Lógica simplificada de clique
  async filtrarPor(coluna: string) {
    try {
      const atual = this.ordenacaoAtual();

      // Se clicou na mesma coluna, inverte a ordem. Se clicou em outra, padrão é decrescente (true)
      const desc = atual.coluna === coluna ? !atual.desc : true;

      // Salva o novo estado visual
      this.ordenacaoAtual.set({ coluna, desc });

      // Busca no SQLite passando as variáveis corretas
      const dados = await obterProcessosAgrupados(coluna, desc);
      this.processos_agrupados.set(dados);
    } catch (erro) {
      console.error("Erro ao buscar dados agrupados:", erro);
    }
  }
}
