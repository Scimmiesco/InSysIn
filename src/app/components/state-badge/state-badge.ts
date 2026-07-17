import { Component, input } from "@angular/core";
import { LowerCasePipe } from "@angular/common";

@Component({
  selector: "ui-state-badge",
  standalone: true,
  imports: [LowerCasePipe],
  template: `
    <span
      class="state-badge"
      [class]="'st-' + (state() | lowercase)"
      [attr.data-tip]="tooltip()"
    >{{ state() || "—" }}</span>
  `,
  styles: `
    :host { display: inline-block; }
    .state-badge {
      display: inline-block;
      padding: 1px 6px;
      border-radius: 8px;
      font-size: 9px;
      font-weight: 600;
      letter-spacing: 0.3px;
    }
    .st-close_wait, .st-time_wait { background: rgba(210, 153, 34, 0.15); color: var(--color-warning); }
    .st-established { background: rgba(63, 185, 80, 0.15); color: var(--color-success); }
    .st-listen { background: rgba(88, 166, 255, 0.15); color: var(--color-info); }
    .st-syn_sent, .st-syn_received { background: rgba(210, 153, 34, 0.15); color: var(--color-warning); }
    .st-fin_wait_1, .st-fin_wait_2, .st-closing, .st-last_ack, .st-closed { background: rgba(139, 148, 158, 0.12); color: var(--color-muted-state); }
    .st-close, .st-time_wait { background: rgba(210, 153, 34, 0.15); color: var(--color-warning); }
  `,
})
export class StateBadgeComponent {
  readonly state = input.required<string>();
  readonly tooltip = input<string | null>(null);
}
