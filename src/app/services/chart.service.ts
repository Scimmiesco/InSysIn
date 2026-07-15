import { Injectable } from '@angular/core';
import { MemReading, ChartLabel } from '../models/chart.model';

@Injectable({ providedIn: 'root' })
export class ChartService {
  calcularViewBox(readings: MemReading[]): string {
    const n = Math.max(readings.length, 2);
    return `0 0 ${n} 100`;
  }

  calcularLinePoints(readings: MemReading[]): string {
    return readings
      .map((r, i) => `${i},${100 - r.pct * 100}`)
      .join(' ');
  }

  calcularStddevPath(readings: MemReading[]): string {
    const vals = readings.map((r) => r.pct);
    const n = vals.length;
    if (n < 3) return '';
    const mean = vals.reduce((a, b) => a + b, 0) / n;
    const variance = vals.reduce((sum, v) => sum + (v - mean) ** 2, 0) / n;
    const stddev = Math.sqrt(variance);
    let path = '';
    for (let i = 0; i < n; i++) {
      const y = 100 - Math.min((mean + stddev) * 100, 100);
      path += `${i === 0 ? 'M' : 'L'} ${i},${y}`;
    }
    for (let i = n - 1; i >= 0; i--) {
      const y = 100 - Math.max((mean - stddev) * 100, 0);
      path += ` L ${i},${y}`;
    }
    return path + ' Z';
  }

  calcularLabels(readings: MemReading[]): ChartLabel[] {
    const n = readings.length;
    if (n === 0) return [];
    const count = Math.min(6, n);
    const step = (n - 1) / Math.max(count - 1, 1);
    const labels: ChartLabel[] = [];
    for (let i = 0; i < count; i++) {
      const idx = Math.round(i * step);
      labels.push({
        xPct: ((idx / (n - 1)) * 100).toFixed(1),
        time: readings[idx].time.toLocaleTimeString('pt-BR', {
          hour: '2-digit',
          minute: '2-digit',
        }),
      });
    }
    return labels;
  }
}
