import { Component, input } from "@angular/core";

export type SkeletonVariant = "card" | "row" | "block" | "text";

@Component({
  selector: "ui-skeleton",
  standalone: true,
  template: `
    @for (i of items(); track i) {
      <div class="skeleton" [class.skel-card]="variant() === 'card'" [class.skel-row]="variant() === 'row'">
        @if (variant() === "row") {
          <div class="skel-inner" [style.width.%]="rowWidth(i)"></div>
        }
      </div>
    }
  `,
  styles: `
    :host { display: contents; }
    .skeleton {
      background: var(--bg-elevated);
      border-radius: var(--r-card-sm);
      animation: skeleton-pulse 1.5s ease-in-out infinite;
    }
    .skel-card {
      height: 64px;
    }
    .skel-row {
      height: 28px;
      display: flex;
      align-items: center;
      padding: 0 8px;
    }
    .skel-inner {
      height: 12px;
      background: var(--border-default);
      border-radius: var(--r-bar);
    }
  `,
})
export class SkeletonComponent {
  readonly repeat = input(1, { transform: (v: number) => Math.max(1, v) });
  readonly variant = input<SkeletonVariant>("card");

  protected items(): number[] {
    return Array.from({ length: this.repeat() }, (_, i) => i);
  }

  protected rowWidth(i: number): number {
    const widths = [80, 55, 70, 45, 60, 75, 50, 65];
    return widths[i % widths.length];
  }
}
