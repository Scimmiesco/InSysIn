import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NetworkStore } from '../../store/network.store';
import { NetworkInterface, NetConnection, ListeningService, ProcessConnection } from '../../../generated/types';

@Component({
  selector: 'app-network',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './network.html',
  styleUrl: './network.css',
})
export class Network {
  constructor(protected store: NetworkStore) {}

  rateValue(iface: NetworkInterface, direction: 'rx' | 'tx'): string {
    const bytes = direction === 'rx'
      ? (this.store.ifaceRxRate()[iface.name] ?? 0)
      : (this.store.ifaceTxRate()[iface.name] ?? 0);
    return this.formatBytes(bytes) + '/s';
  }

  formatTotal(bytes: number): string {
    return this.formatBytes(bytes);
  }

  interfaceTip(iface: NetworkInterface): string {
    return `${iface.name} — ${this.interfaceDescription(iface.name)}\nIP: ${iface.ip_addresses.join(', ') || 'none'}`;
  }

  rateRxTip(iface: NetworkInterface): string {
    return `Receiving ${this.rateValue(iface, 'rx')} — data being downloaded through ${iface.name}`;
  }

  rateTxTip(iface: NetworkInterface): string {
    return `Transmitting ${this.rateValue(iface, 'tx')} — data being uploaded through ${iface.name}`;
  }

  connTip(conn: NetConnection): string {
    let tip = `${conn.process_name} (PID ${conn.pid})\nProtocol: ${conn.protocol}\n`;
    tip += `Local: ${conn.local}\n`;
    if (conn.remote) {
      const port = this.portName(conn.remote);
      tip += `Remote: ${conn.remote} (${port})\n`;
    }
    tip += `Status: ${this.stateDescription(conn.state)}`;
    return tip;
  }

  svcTip(svc: ListeningService): string {
    return `${svc.process_name} (PID ${svc.pid}) — ${svc.port_desc}\nListening on ${svc.local}`;
  }

  procTip(pc: ProcessConnection): string {
    const parts: string[] = [];
    if (pc.tcp_count > 0) parts.push(`${pc.tcp_count} TCP`);
    if (pc.udp_count > 0) parts.push(`${pc.udp_count} UDP`);
    return `${pc.process_name} — ${pc.count} connection${pc.count !== 1 ? 's' : ''} (${parts.join(', ')})`;
  }

  listPcs(pc: ProcessConnection): string[] {
    const r: string[] = [];
    if (pc.tcp_count > 0) r.push(`${pc.tcp_count} TCP`);
    if (pc.udp_count > 0) r.push(`${pc.udp_count} UDP`);
    return r;
  }

  private formatBytes(bytes: number): string {
    if (bytes <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    return (bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1) + ' ' + units[i];
  }

  private interfaceDescription(name: string): string {
    const n = name.toLowerCase();
    if (n.startsWith('en')) return 'Built-in network adapter (Ethernet or Wi-Fi). Handles your internet connection.';
    if (n === 'lo0') return 'Loopback interface. Communication between processes on your own machine (localhost).';
    if (n.startsWith('awdl')) return 'Apple Wireless Direct Link. Powers AirDrop, AirPlay, and Continuity features.';
    if (n.startsWith('llw')) return 'Low-Latency WLAN. Apple peer-to-peer connectivity for real-time services.';
    if (n.startsWith('utun')) return 'Virtual tunnel interface. Typically created by VPN software.';
    if (n.startsWith('bridge')) return 'Network bridge. Used when sharing internet or connecting virtual machines.';
    if (n.startsWith('p2p')) return 'Peer-to-peer interface. Related to Apple Wireless Direct Link services.';
    return 'Network interface for data transmission.';
  }

  private portName(addr: string): string {
    const port = parseInt(addr.split(':').pop() || '', 10);
    const services: Record<number, string> = {
      80: 'HTTP — web traffic',
      443: 'HTTPS — secure web traffic',
      22: 'SSH — secure remote access',
      53: 'DNS — domain name resolution',
      25: 'SMTP — email sending',
      587: 'SMTP — email submission',
      993: 'IMAPS — secure email',
      3306: 'MySQL database',
      5432: 'PostgreSQL database',
      6379: 'Redis cache',
      3000: 'Development server',
      8080: 'HTTP alternate',
      5353: 'mDNS — Bonjour service discovery',
      123: 'NTP — time synchronization',
    };
    return services[port] || `port ${port}`;
  }

  private stateDescription(state: string): string {
    switch (state) {
      case 'ESTABLISHED': return 'actively connected — data is flowing between both ends';
      case 'LISTEN': return 'waiting for incoming connections (server mode)';
      case 'CLOSE_WAIT': return 'remote side has closed — waiting to finish closing';
      case 'TIME_WAIT': return 'connection closed — keeping state briefly for delayed packets';
      case 'SYN_SENT': return 'attempting to establish a connection — not yet complete';
      default: return state ? state.toLowerCase().replace(/_/g, ' ') : 'no status';
    }
  }
}
