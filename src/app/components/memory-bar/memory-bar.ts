import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MemInfo } from '../../../generated/types';

@Component({
  selector: 'app-memory-bar',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './memory-bar.html',
  styleUrl: './memory-bar.css',
})
export class MemoryBar {
  @Input({ required: true }) memInfo!: MemInfo;

  totalGB(): number { return (this.memInfo?.total_memory ?? 0) / 1_073_741_824; }
  freeGB(): number { return (this.memInfo?.free_memory ?? 0) / 1_073_741_824; }
  usedGB(): number { return (this.memInfo?.used_memory ?? 0) / 1_073_741_824; }

  appGB(): number { return ((this.memInfo?.breakdown?.app_memory ?? 0)) / 1_073_741_824; }
  wiredGB(): number { return ((this.memInfo?.breakdown?.wired_memory ?? 0)) / 1_073_741_824; }
  compressedGB(): number { return ((this.memInfo?.breakdown?.compressed_memory ?? 0)) / 1_073_741_824; }
  cachedGB(): number {
    if (this.memInfo?.breakdown) {
      return ((this.memInfo.breakdown.cached_memory)) / 1_073_741_824;
    }
    const avail = (this.memInfo?.available_memory ?? 0) / 1_073_741_824;
    return avail - this.freeGB();
  }

  hasBreakdown(): boolean { return !!this.memInfo?.breakdown; }

  pct(v: number): number {
    const t = this.totalGB();
    return t > 0 ? (v / t) * 100 : 0;
  }
}
