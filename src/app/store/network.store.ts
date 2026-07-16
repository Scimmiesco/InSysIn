import { Injectable, OnDestroy, signal } from '@angular/core';
import { NetworkDashboard, NetConnection, NetworkInterface } from '../../generated/types';
import { NetworkService } from '../services/network.service';

export interface ProxyEntry {
  id: string;
  timestamp: Date;
  protocol: string;
  direction: 'outgoing' | 'incoming' | 'closed';
  process_name: string;
  pid: number;
  local: string;
  remote: string;
  state: string;
  hostname?: string;
}

@Injectable({ providedIn: 'root' })
export class NetworkStore implements OnDestroy {
  dashboard = signal<NetworkDashboard | null>(null);
  error = signal<string | null>(null);

  activeTab = signal<'dashboard' | 'proxy'>('dashboard');
  proxyLog = signal<ProxyEntry[]>([]);
  private prevConns = new Map<string, NetConnection>();

  ifaceRxRate = signal<Record<string, number>>({});
  ifaceTxRate = signal<Record<string, number>>({});
  private prevRx: Record<string, number> = {};
  private prevTx: Record<string, number> = {};

  private pollingTimer?: ReturnType<typeof setInterval>;
  private readonly POLL_MS = 3000;
  private readonly MAX_PROXY = 500;

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

  switchTab(tab: 'dashboard' | 'proxy'): void {
    this.activeTab.set(tab);
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

      if (res?.connections) {
        this.updateProxyLog(res.connections);
      }

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

  private connKey(conn: NetConnection): string {
    return `${conn.pid}:${conn.protocol}:${conn.local}:${conn.remote}`;
  }

  private updateProxyLog(current: NetConnection[]): void {
    const curMap = new Map<string, NetConnection>();
    const fresh: ProxyEntry[] = [];

    for (const conn of current) {
      const key = this.connKey(conn);
      curMap.set(key, conn);

      if (!this.prevConns.has(key)) {
        fresh.push({
          id: key + Date.now(),
          timestamp: new Date(),
          protocol: conn.protocol,
          direction: conn.remote ? 'outgoing' : 'incoming',
          process_name: conn.process_name,
          pid: conn.pid,
          local: conn.local,
          remote: conn.remote,
          state: conn.state,
          hostname: (conn as any).hostname,
        });
      }
    }

    for (const [key, conn] of this.prevConns) {
      if (!curMap.has(key)) {
        fresh.push({
          id: key + Date.now() + 'x',
          timestamp: new Date(),
          protocol: conn.protocol,
          direction: 'closed',
          process_name: conn.process_name,
          pid: conn.pid,
          local: conn.local,
          remote: conn.remote,
          state: conn.state,
          hostname: (conn as any).hostname,
        });
      }
    }

    this.prevConns = curMap;

    if (fresh.length > 0) {
      this.proxyLog.update((log) => {
        const next = [...fresh, ...log];
        return next.length > this.MAX_PROXY ? next.slice(0, this.MAX_PROXY) : next;
      });
    }
  }
}
