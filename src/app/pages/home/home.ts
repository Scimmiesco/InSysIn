import { Component } from '@angular/core';
import { DecimalPipe, CommonModule } from '@angular/common';
import { GaugeCard } from '../../components/gauge-card/gauge-card';
import { LiveBadgeComponent } from '../../components/live-badge/live-badge';
import { SectionComponent } from '../../components/section/section';
import { SystemStore } from '../../store/system.store';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [DecimalPipe, CommonModule, GaugeCard, LiveBadgeComponent, SectionComponent],
  templateUrl: './home.html',
  styleUrl: './home.css',
})
export class Home {
  constructor(protected store: SystemStore) {}

  get ramPct(): number {
    const m = this.store.sys_info()?.mem_info;
    if (!m) return 0;
    return (m.used_memory / m.total_memory) * 100;
  }

  get diskPct(): number {
    const d = this.store.sys_info()?.disk_usage;
    if (!d || !d.total_bytes) return 0;
    return ((d.total_bytes - d.available_bytes) / d.total_bytes) * 100;
  }

  get memBreakdown(): {
    app: number; wired: number; compressed: number; cached: number;
    appPct: number; wiredPct: number; compressedPct: number; cachedPct: number;
  } | null {
    const m = this.store.sys_info()?.mem_info;
    if (!m) return null;
    const total = m.total_memory || 1;
    const b = m.breakdown;
    const app = b?.app_memory ?? 0;
    const wired = b?.wired_memory ?? 0;
    const compressed = b?.compressed_memory ?? 0;
    const cached = b?.cached_memory ?? 0;
    return {
      app, wired, compressed, cached,
      appPct: (app / total) * 100,
      wiredPct: (wired / total) * 100,
      compressedPct: (compressed / total) * 100,
      cachedPct: (cached / total) * 100,
    };
  }

  memGbs(bytes: number): string {
    return (bytes / 1_073_741_824).toFixed(1) + ' GB';
  }

  get swapUsed(): number {
    return this.store.sys_info()?.mem_info?.used_swap ?? 0;
  }

  get swapTotal(): number {
    return this.store.sys_info()?.mem_info?.total_swap ?? 0;
  }

  get cpuHistoryPoints(): string {
    return this.buildSparkline('cpuPct');
  }

  get ramHistoryPoints(): string {
    return this.buildSparkline('ramPct');
  }

  private buildSparkline(field: 'cpuPct' | 'ramPct'): string {
    const readings = this.store.memReadings();
    if (readings.length < 2) return '';
    const W = 300;
    const H = 60;
    const step = Math.max(1, Math.floor(readings.length / 90));
    const points: string[] = [];
    for (let i = 0; i < readings.length; i += step) {
      const x = (i / (readings.length - 1)) * W;
      const val = Math.min(100, Math.max(0, readings[i][field] * 100));
      const y = H - (val / 100) * H;
      points.push(`${x.toFixed(1)},${y.toFixed(1)}`);
    }
    if (points.length < 2) return '';
    return points.join(' ');
  }

  get historyCount(): number {
    return this.store.memReadings().length;
  }

  get diskReadRate(): string {
    return `${this.store.diskReadRate()} MB/s`;
  }

  get diskWriteRate(): string {
    return `${this.store.diskWriteRate()} MB/s`;
  }

  get netDownRate(): string {
    return `${this.store.netDownloadRate()} MB/s`;
  }

  get netUpRate(): string {
    return `${this.store.netUploadRate()} MB/s`;
  }

  get uptime(): string {
    const s = this.store.sys_info()?.system_info?.uptime_secs ?? 0;
    const d = Math.floor(s / 86400);
    const h = Math.floor((s % 86400) / 3600);
    const m = Math.floor((s % 3600) / 60);
    const parts: string[] = [];
    if (d > 0) parts.push(`${d}d`);
    if (h > 0) parts.push(`${h}h`);
    parts.push(`${m}m`);
    return parts.join(' ');
  }
}
