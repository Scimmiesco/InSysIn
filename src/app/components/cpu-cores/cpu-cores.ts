import { Component, Input } from '@angular/core';
import { DecimalPipe, CommonModule } from '@angular/common';
import { SectionComponent } from '../section/section';
import type { CoreInfo, GpuInfo, SystemInfo } from '../../../generated/types';

@Component({
  selector: 'app-cpu-cores',
  standalone: true,
  imports: [DecimalPipe, CommonModule, SectionComponent],
  templateUrl: './cpu-cores.html',
  styleUrl: './cpu-cores.css',
})
export class CpuCoresComponent {
  @Input({ required: true }) cores!: CoreInfo[];
  @Input({ required: true }) systemInfo!: SystemInfo;
  @Input() cpuUsage = 0;
  @Input() cpuTemperature: number | null = null;
  @Input() cpuUser = 0;
  @Input() cpuSystem = 0;
  @Input() gpu: GpuInfo | null = null;

  get pCores(): number {
    return this.systemInfo.performance_cores;
  }

  get eCores(): number {
    return this.systemInfo.efficiency_cores;
  }

  get totalCores(): number {
    return this.pCores + this.eCores;
  }

  get tempDisplay(): string {
    if (this.cpuTemperature == null) return '--';
    return this.cpuTemperature.toFixed(0) + '°C';
  }

  get physicalCores(): CoreInfo[] {
    return this.cores.filter(c => c.physical);
  }
}
