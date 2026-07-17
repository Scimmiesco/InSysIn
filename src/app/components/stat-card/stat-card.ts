import { Component, input } from "@angular/core";

export type StatVariant = "default" | "success" | "info" | "warning";

@Component({
  selector: "ui-stat-card",
  standalone: true,
  template: `
    <div class="stat-card" [attr.data-tip]="tooltip()">
      <span class="stat-value" [class]="'stat-' + variant()">{{ value() }}</span>
      <span class="stat-label">{{ label() }}</span>
    </div>
  `,
  styles: `
    :host { display: block; }
    .stat-card {
      background: var(--bg-card);
      border: 1px solid var(--border-default);
      border-radius: var(--r-card-sm);
      padding: 8px 10px;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 2px;
      cursor: help;
    }
    .stat-value {
      font-size: 20px;
      font-weight: 700;
      color: var(--text-primary);
      font-variant-numeric: tabular-nums;
    }
    .stat-success { color: var(--color-success); }
    .stat-info { color: var(--color-info); }
    .stat-warning { color: var(--color-warning); }
    .stat-label {
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: var(--text-muted);
      font-weight: 600;
    }
  `,
})
export class StatCardComponent {
  readonly value = input.required<number | string>();
  readonly label = input.required<string>();
  readonly variant = input<StatVariant>("default");
  readonly tooltip = input<string | null>(null);
}
