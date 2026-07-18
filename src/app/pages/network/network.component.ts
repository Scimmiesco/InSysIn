import { Component, inject } from "@angular/core";
import { CommonModule } from "@angular/common";
import { NetworkStore, ProxyEntry } from "../../store/network.store";
import { NetworkInterface, NetConnection, ListeningService, ProcessConnection } from "../../../generated/types";
import { LiveBadgeComponent } from "../../components/live-badge/live-badge";
import { SectionComponent } from "../../components/section/section";
import { StatCardComponent } from "../../components/stat-card/stat-card";
import { StateBadgeComponent } from "../../components/state-badge/state-badge";
import { ButtonComponent } from "../../components/button/button";

@Component({
  selector: "app-network",
  standalone: true,
  imports: [CommonModule, LiveBadgeComponent, SectionComponent, StatCardComponent, StateBadgeComponent, ButtonComponent],
  templateUrl: "./network.component.html",
  styleUrl: "./network.component.css",
})
export class Network {
  protected store = inject(NetworkStore);

  get filteredConnections(): NetConnection[] {
    const conns = this.store.dashboard()?.connections ?? [];
    const f = this.store.connectionFilter().toLowerCase().trim();
    if (!f) return conns;
    return conns.filter(c =>
      c.process_name.toLowerCase().includes(f) ||
      c.protocol.toLowerCase().includes(f) ||
      c.local.toLowerCase().includes(f) ||
      c.remote.toLowerCase().includes(f) ||
      String(c.pid).includes(f)
    );
  }  
  constructor() {}

  rateValue(iface: NetworkInterface, direction: "rx" | "tx"): string {
    const bytes = direction === "rx"
      ? (this.store.ifaceRxRate()[iface.name] ?? 0)
      : (this.store.ifaceTxRate()[iface.name] ?? 0);
    return this.formatBytes(bytes) + "/s";
  }

  formatTotal(bytes: number): string {
    return this.formatBytes(bytes);
  }

  interfaceTip(iface: NetworkInterface): string {
    return `${iface.name} — ${this.interfaceDescription(iface.name)}\nIP: ${iface.ip_addresses.join(", ") || "none"}`;
  }

  rateRxTip(iface: NetworkInterface): string {
    return `Receiving ${this.rateValue(iface, "rx")} — data being downloaded through ${iface.name}`;
  }

  rateTxTip(iface: NetworkInterface): string {
    return `Transmitting ${this.rateValue(iface, "tx")} — data being uploaded through ${iface.name}`;
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
    return `${pc.process_name} — ${pc.count} connection${pc.count !== 1 ? "s" : ""} (${parts.join(", ")})`;
  }

  listPcs(pc: ProcessConnection): string[] {
    const r: string[] = [];
    if (pc.tcp_count > 0) r.push(`${pc.tcp_count} TCP`);
    if (pc.udp_count > 0) r.push(`${pc.udp_count} UDP`);
    return r;
  }

  proxyTime(e: ProxyEntry): string {
    const d = e.timestamp;
    return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  }

  proxyRemote(e: ProxyEntry): string {
    const host = e.hostname || e.remote || e.local;
    const addr = e.remote || e.local;
    const port = this.portName(addr);
    if (e.hostname && e.remote) {
      const p = port.startsWith("port ") ? "" : ` (${port})`;
      return `${e.hostname}:${e.remote.split(":").pop()}${p}`;
    }
    return port.startsWith("port ") ? addr : `${addr} (${port})`;
  }

  proxyTip(e: ProxyEntry): string {
    const dir = e.direction === "outgoing" ? "Outgoing connection" : e.direction === "incoming" ? "Incoming (listening)" : "Connection closed";
    let tip = `${e.process_name} (PID ${e.pid})\n${dir}\n`;
    tip += `Protocol: ${e.protocol}\n`;
    if (e.remote) {
      tip += `Remote: ${this.proxyRemote(e)}\n`;
    }
    tip += `State: ${this.stateDescription(e.state)}`;
    return tip;
  }

  stateTip(state: string): string {
    if (!state) return "";
    switch (state) {
      case "ESTABLISHED": return "Connection is active — data flows freely between your computer and the server.";
      case "LISTEN": return "Waiting for incoming connections — this app acts as a server on this port.";
      case "SYN_SENT": return "Trying to reach the server — waiting for a reply. May be slow or blocked.";
      case "SYN_RECEIVED": return "Server received your connection request — finalizing handshake.";
      case "CLOSE_WAIT": return "The remote server has disconnected — waiting to close this end.";
      case "TIME_WAIT": return "Connection recently closed — holding briefly to catch delayed packets.";
      case "FIN_WAIT_1": return "Starting to close the connection — sent the shutdown request.";
      case "FIN_WAIT_2": return "The remote end acknowledged the close — waiting for its final packet.";
      case "CLOSING": return "Both sides are closing at the same time.";
      case "LAST_ACK": return "The server is waiting for the final acknowledgment before closing.";
      case "CLOSED": return "Connection has fully closed.";
      default: return "";
    }
  }

  private formatBytes(bytes: number): string {
    if (bytes <= 0) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    return (bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1) + " " + units[i];
  }

  private interfaceDescription(name: string): string {
    const n = name.toLowerCase();
    if (n.startsWith("en")) return "Built-in network adapter (Ethernet or Wi-Fi). Handles your internet connection.";
    if (n === "lo0") return "Loopback interface. Communication between processes on your own machine (localhost).";
    if (n.startsWith("awdl")) return "Apple Wireless Direct Link. Powers AirDrop, AirPlay, and Continuity features.";
    if (n.startsWith("llw")) return "Low-Latency WLAN. Apple peer-to-peer connectivity for real-time services.";
    if (n.startsWith("utun")) return "Virtual tunnel interface. Typically created by VPN software.";
    if (n.startsWith("bridge")) return "Network bridge. Used when sharing internet or connecting virtual machines.";
    if (n.startsWith("p2p")) return "Peer-to-peer interface. Related to Apple Wireless Direct Link services.";
    return "Network interface for data transmission.";
  }

  private portName(addr: string): string {
    const port = parseInt(addr.split(":").pop() || "", 10);
    const services: Record<number, string> = {
      80: "HTTP",
      443: "HTTPS",
      22: "SSH",
      53: "DNS",
      25: "SMTP",
      993: "IMAPS",
      3306: "MySQL",
      5432: "PostgreSQL",
      6379: "Redis",
      8080: "HTTP-alt",
      5353: "mDNS",
      123: "NTP",
    };
    return services[port] || `port ${port}`;
  }

  private stateDescription(state: string): string {
    switch (state) {
      case "ESTABLISHED": return "actively connected — data is flowing between both ends";
      case "LISTEN": return "waiting for incoming connections (server mode)";
      case "CLOSE_WAIT": return "remote side has closed — waiting to finish closing";
      case "TIME_WAIT": return "connection closed — keeping state briefly for delayed packets";
      case "SYN_SENT": return "attempting to establish a connection — not yet complete";
      default: return state ? state.toLowerCase().replace(/_/g, " ") : "no status";
    }
  }
}

function pad(n: number): string {
  return n < 10 ? "0" + n : "" + n;
}
