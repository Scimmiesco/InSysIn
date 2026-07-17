import { Component, input } from "@angular/core";

@Component({
  selector: "ui-info-item",
  standalone: true,
  template: `
    <div class="info-item" [attr.data-tip]="tooltip()">
      <span class="info-key">{{ key() }}</span>
      <span class="info-value" [class.mono]="mono()">{{ value() || "—" }}</span>
      @if (subtitle()) {
        <span class="info-subtitle">{{ subtitle() }}</span>
      }
    </div>
  `,
  styles: `
    :host { display: block; }
    .info-item {
      background: var(--bg-card);
      border: 1px solid var(--border-default);
      border-radius: var(--r-card-sm);
      padding: var(--sp-card);
      display: flex;
      flex-direction: column;
      gap: 4px;
      cursor: help;
    }
    .info-key {
      font-size: var(--fs-xs);
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.4px;
      color: var(--text-muted);
    }
    .info-value {
      font-size: var(--fs-base);
      font-weight: 600;
      color: var(--text-primary);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .info-value.mono {
      font-family: var(--font-mono);
    }
    .info-subtitle {
      font-size: var(--fs-xs);
      color: var(--text-muted);
    }
  `,
})
export class InfoItemComponent {
  readonly key = input.required<string>();
  readonly value = input.required<string>();
  readonly mono = input(false);
  readonly tooltip = input<string | null>(null);
  readonly subtitle = input<string | null>(null);
}
