import { Component, input } from "@angular/core";

@Component({
  selector: "ui-section",
  standalone: true,
  template: `
    <div class="section-box">
      <div class="section-title">
        <span class="section-title-text">{{ title() }}</span>
        @if (subtitle()) {
          <span class="section-sub">{{ subtitle() }}</span>
        }
        <span class="section-spacer"></span>
        <ng-content select="[header-actions]" />
      </div>
      <ng-content />
    </div>
  `,
  styles: `
    :host { display: block; }
    .section-box {
      border: 1px solid var(--border-default);
      border-radius: var(--r-card-sm);
      padding: var(--sp-card);
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    .section-title {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: var(--fs-sm);
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: var(--text-primary);
    }
    .section-title-text {
      flex-shrink: 0;
    }
    .section-spacer {
      flex: 1;
    }
    .section-sub {
      font-weight: 400;
      text-transform: none;
      letter-spacing: 0;
      color: var(--text-muted);
      font-size: 11px;
    }
  `,
})
export class SectionComponent {
  readonly title = input.required<string>();
  readonly subtitle = input<string | null>(null);
}
