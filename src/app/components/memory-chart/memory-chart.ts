import { Component, Input, computed } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MemReading } from '../../models/chart.model';
import { ChartService } from '../../services/chart.service';

@Component({
  selector: 'app-memory-chart',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './memory-chart.html',
  styleUrl: './memory-chart.css',
})
export class MemoryChart {
  @Input({ required: true }) readings!: MemReading[];

  constructor(private chart: ChartService) {}

  protected viewBox = computed(() => this.chart.calcularViewBox(this.readings));
  protected linePoints = computed(() => this.chart.calcularLinePoints(this.readings));
  protected stddevPath = computed(() => this.chart.calcularStddevPath(this.readings));
  protected labels = computed(() => this.chart.calcularLabels(this.readings));
}
