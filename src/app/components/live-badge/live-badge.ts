import { Component, input } from "@angular/core";

@Component({
  selector: "ui-live-badge",
  standalone: true,
  template: `
    <span class="live-badge">
      <span class="live-dot"></span>
      {{ label() }}
    </span>
  `,
  styles: `
    :host { display: inline-flex; }
    .live-badge {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: var(--fs-sm);
      font-weight: 600;
      color: var(--accent);
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    .live-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--accent);
      animation: pulse 2s ease-in-out infinite;
    }
  `,
})
export class LiveBadgeComponent {
  readonly label = input("Live");
}
