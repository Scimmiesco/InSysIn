import { Injectable, OnDestroy, signal } from '@angular/core';
import { NetworkDashboard, NetworkInterface } from '../../generated/types';
import { NetworkService } from '../services/network.service';

@Injectable({ providedIn: 'root' })
export class NetworkStore implements OnDestroy {
  dashboard = signal<NetworkDashboard | null>(null);
  error = signal<string | null>(null);

  ifaceRxRate = signal<Record<string, number>>({});
  ifaceTxRate = signal<Record<string, number>>({});
  private prevRx: Record<string, number> = {};
  private prevTx: Record<string, number> = {};

  private pollingTimer?: ReturnType<typeof setInterval>;
  private readonly POLL_MS = 3000;

  constructor(private network: NetworkService) {
    this.iniciar();
  }

  private iniciar(): void {
    this.lerDados();
    this.pollingTimer = setInterval(() => this.lerDados(), this.POLL_MS);
  }

  ngOnDestroy(): void {
    if (this.pollingTimer) clearInterval(this.pollingTimer);
  }

  allInterfaces(): NetworkInterface[] {
    const t = this.dashboard()?.traffic;
    if (!t) return [];
    return [...t.physical, ...t.virtual_ifaces, ...t.special];
  }

  async lerDados(): Promise<void> {
    try {
      const res = await this.network.carregar();
      this.dashboard.set(res);
      this.error.set(null);

      if (res?.traffic) {
        const sec = this.POLL_MS / 1000;
        const newRx: Record<string, number> = {};
        const newTx: Record<string, number> = {};

        for (const iface of this.allInterfaces()) {
          const prevRx = this.prevRx[iface.name] ?? 0;
          const prevTx = this.prevTx[iface.name] ?? 0;
          newRx[iface.name] = prevRx > 0
            ? Math.round((iface.received_bytes - prevRx) / sec)
            : 0;
          newTx[iface.name] = prevTx > 0
            ? Math.round((iface.transmitted_bytes - prevTx) / sec)
            : 0;
          this.prevRx[iface.name] = iface.received_bytes;
          this.prevTx[iface.name] = iface.transmitted_bytes;
        }
        this.ifaceRxRate.set(newRx);
        this.ifaceTxRate.set(newTx);
      }
    } catch (erro) {
      this.error.set('Erro ao ler dados de rede');
      console.error('Erro ao ler rede:', erro);
    }
  }
}
