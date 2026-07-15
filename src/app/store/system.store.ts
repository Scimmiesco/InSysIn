import { Injectable, OnDestroy, signal } from '@angular/core';
import { SysStats, HistoricoCompleto, ProcessoAgrupado } from '../../generated/types';
import { MemReading } from '../models/chart.model';
import { HardwareService } from '../services/hardware.service';
import { HistoryService } from '../services/history.service';

@Injectable({ providedIn: 'root' })
export class SystemStore implements OnDestroy {
  sys_info = signal<SysStats | null>(null);
  sys_history = signal<HistoricoCompleto | null>(null);
  processos_agrupados = signal<ProcessoAgrupado[]>([]);
  ordenacaoAtual = signal<{ coluna: string; desc: boolean }>({ coluna: 'memoria', desc: true });
  memReadings = signal<MemReading[]>([]);
  error = signal<string | null>(null);

  private pollingTimer?: ReturnType<typeof setInterval>;
  private histTimer?: ReturnType<typeof setInterval>;

  private readonly POLL_MS = 1000;
  private readonly HIST_MS = 10000;
  private readonly MAX_READINGS = 180;

  constructor(
    private hardware: HardwareService,
    private history: HistoryService,
  ) {
    this.iniciar();
  }

  private iniciar(): void {
    this.lerDados();
    this.obterHistorico();
    this.filtrarPor('memoria');

    this.pollingTimer = setInterval(() => this.lerDados(), this.POLL_MS);
    this.histTimer = setInterval(() => this.obterHistorico(), this.HIST_MS);
  }

  ngOnDestroy(): void {
    if (this.pollingTimer) clearInterval(this.pollingTimer);
    if (this.histTimer) clearInterval(this.histTimer);
  }

  async lerDados(): Promise<void> {
    try {
      const res = await this.hardware.carregar();
      this.sys_info.set(res);
      this.error.set(null);
      if (res?.mem_info) {
        const pct = res.mem_info.used_memory / res.mem_info.total_memory;
        this.memReadings.update((prev) => {
          const next = [...prev, { pct, time: new Date() }];
          return next.length > this.MAX_READINGS
            ? next.slice(next.length - this.MAX_READINGS)
            : next;
        });
      }
    } catch (erro) {
      this.error.set('Erro ao ler dados do sistema');
      console.error('Erro ao ler hardware:', erro);
    }
  }

  async obterHistorico(): Promise<void> {
    try {
      const res = await this.history.carregar();
      this.sys_history.set(res);
    } catch (erro) {
      console.error('Erro ao obter histórico:', erro);
    }
  }

  async filtrarPor(coluna: string): Promise<void> {
    try {
      const atual = this.ordenacaoAtual();
      const desc = atual.coluna === coluna ? !atual.desc : true;
      this.ordenacaoAtual.set({ coluna, desc });
      const dados = await this.history.carregarAgrupados(coluna, desc);
      this.processos_agrupados.set(dados);
    } catch (erro) {
      console.error('Erro ao buscar dados agrupados:', erro);
    }
  }
}
