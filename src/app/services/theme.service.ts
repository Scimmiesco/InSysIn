import { Injectable, signal } from '@angular/core';
import { listen } from '@tauri-apps/api/event';

type Theme = 'dark' | 'light';

@Injectable({ providedIn: 'root' })
export class ThemeService {
  current = signal<Theme>('dark');

  constructor() {
    const saved = localStorage.getItem('theme') as Theme | null;
    if (saved === 'dark' || saved === 'light') {
      this.apply(saved);
    } else {
      this.apply('dark');
    }
    this.listenMenu();
  }

  toggle(): void {
    this.apply(this.current() === 'dark' ? 'light' : 'dark');
  }

  private apply(theme: Theme): void {
    document.documentElement.classList.remove('dark', 'light');
    document.documentElement.classList.add(theme);
    this.current.set(theme);
    localStorage.setItem('theme', theme);
  }

  private listenMenu(): void {
    try {
      listen<string>('theme-changed', (e) => {
        const t = e.payload as Theme;
        if (t === 'dark' || t === 'light') this.apply(t);
      });
    } catch {}
  }
}
