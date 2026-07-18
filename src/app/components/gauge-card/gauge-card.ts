import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-gauge-card',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './gauge-card.html',
  styleUrl: './gauge-card.css',
})
export class GaugeCard {
  @Input({ required: true }) label!: string;
  @Input({ required: true }) percent!: number;
  @Input() subtitle = '';
  @Input() color = '#58a6ff';
  @Input() rateTop = '';
  @Input() rateBottom = '';

  private readonly R = 22;
  private readonly C = 2 * Math.PI * this.R;

  circumference = this.C;

  get dashOffset(): number {
    return this.C - (this.percent / 100) * this.C;
  }
}
