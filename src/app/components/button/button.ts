import { Component, input, output } from "@angular/core";

export type ButtonVariant = "default" | "primary" | "tab";

@Component({
  selector: "ui-button",
  standalone: true,
  template: `
    <button
      class="btn"
      [class]="'btn-' + variant()"
      [class.active]="active()"
      [disabled]="disabled()"
      (click)="clicked.emit()">
      <ng-content />
    </button>
  `,
  styles: `
    :host { display: inline-flex; }
    .btn {
      padding: 6px 16px;
      border: 1px solid var(--border-default);
      border-radius: var(--r-card-sm);
      background: var(--bg-elevated);
      color: var(--text-primary);
      font-size: var(--fs-sm);
      font-weight: 600;
      cursor: pointer;
      transition: background 0.15s;
      font-family: var(--font-sans);
      line-height: inherit;
    }
    .btn:hover:not(:disabled) { background: var(--border-default); }
    .btn:disabled { opacity: 0.5; cursor: not-allowed; }

    .btn-primary {
      background: var(--accent);
      border-color: var(--accent);
      color: #fff;
    }
    .btn-primary:hover:not(:disabled) { background: var(--accent-muted); border-color: var(--accent-muted); }

    .btn-tab {
      border: none;
      border-bottom: 2px solid transparent;
      border-radius: 0;
      background: none;
      color: var(--text-muted);
      font-weight: 500;
      padding: 6px 16px;
    }
    .btn-tab:hover { color: var(--text-secondary); }
    .btn-tab.active { color: var(--accent); border-bottom-color: var(--accent); }
  `,
})
export class ButtonComponent {
  readonly variant = input<ButtonVariant>("default");
  readonly disabled = input(false);
  readonly active = input(false);
  readonly clicked = output<void>();
}
