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
