import { Injectable, OnDestroy, signal } from '@angular/core';
import { SysStats, HistoricoCompleto, ProcessoAgrupado } from '../../generated/types';
import { MemReading } from '../models/chart.model';
import { HardwareService } from '../services/hardware.service';
import { HistoryService } from '../services/history.service';

@Injectable({ providedIn: 'root' })
export class SystemStore implements OnDestroy {
  loading = signal(false);
  sys_info = signal<SysStats | null>(null);
  sys_history = signal<HistoricoCompleto | null>(null);
  processos_agrupados = signal<ProcessoAgrupado[]>([]);
  ordenacaoAtual = signal<{ coluna: string; desc: boolean }>({ coluna: 'memoria', desc: true });
  memReadings = signal<MemReading[]>([]);
  error = signal<string | null>(null);

  diskReadRate = signal(0);
  diskWriteRate = signal(0);
  netDownloadRate = signal(0);
  netUploadRate = signal(0);

  private pollingTimer?: ReturnType<typeof setInterval>;
  private histTimer?: ReturnType<typeof setInterval>;

  private readonly POLL_MS = 1000;
  private readonly HIST_MS = 10000;
  private readonly MAX_READINGS = 180;

  private prevDiskRead = 0;
  private prevDiskWrite = 0;
  private prevNetRx = 0;
  private prevNetTx = 0;

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
      this.loading.set(true);
      const res = await this.hardware.carregar();
      this.sys_info.set(res);
      this.error.set(null);

      if (res?.disk_usage) {
        const sec = this.POLL_MS / 1000;
        this.diskReadRate.set(this.prevDiskRead > 0
          ? Math.round((res.disk_usage.read_bytes - this.prevDiskRead) / sec / 1_048_576 * 10) / 10
          : 0);
        this.diskWriteRate.set(this.prevDiskWrite > 0
          ? Math.round((res.disk_usage.write_bytes - this.prevDiskWrite) / sec / 1_048_576 * 10) / 10
          : 0);
        this.prevDiskRead = res.disk_usage.read_bytes;
        this.prevDiskWrite = res.disk_usage.write_bytes;
      }

      if (res?.network_usage) {
        const sec = this.POLL_MS / 1000;
        this.netDownloadRate.set(this.prevNetRx > 0
          ? Math.round((res.network_usage.received_bytes - this.prevNetRx) / sec / 1_048_576 * 10) / 10
          : 0);
        this.netUploadRate.set(this.prevNetTx > 0
          ? Math.round((res.network_usage.transmitted_bytes - this.prevNetTx) / sec / 1_048_576 * 10) / 10
          : 0);
        this.prevNetRx = res.network_usage.received_bytes;
        this.prevNetTx = res.network_usage.transmitted_bytes;
      }

      if (res?.mem_info) {
        const ramPct = res.mem_info.used_memory / res.mem_info.total_memory;
        const cpuPct = (res.cpu_usage ?? 0) / 100;
        this.memReadings.update((prev) => {
          const next = [...prev, { ramPct, cpuPct, time: new Date() }];
          return next.length > this.MAX_READINGS
            ? next.slice(next.length - this.MAX_READINGS)
            : next;
        });
      }
    } catch (erro) {
      this.error.set('Erro ao ler dados do sistema');
      console.error('Erro ao ler hardware:', erro);
    } finally {
      this.loading.set(false);
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
