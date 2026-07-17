import { Injectable, signal } from '@angular/core';
import type { InternetDiagnostics, SpeedResult } from '../../generated/types';
import { WifiService } from '../services/wifi.service';

@Injectable({ providedIn: 'root' })
export class WifiStore {
  diagnostics = signal<InternetDiagnostics | null>(null);
  error = signal<string | null>(null);
  scanning = signal(false);
  runningTest = signal(false);
  speedError = signal<string | null>(null);

  constructor(private wifi: WifiService) {}

  async fetchInfo(): Promise<void> {
    this.scanning.set(true);
    this.error.set(null);
    try {
      const res = await this.wifi.carregar();
      this.diagnostics.set(res);
    } catch (erro) {
      this.error.set('Failed to fetch network info');
      console.error('Internet diagnostics error:', erro);
    } finally {
      this.scanning.set(false);
    }
  }

  async runSpeedTest(): Promise<void> {
    this.runningTest.set(true);
    this.speedError.set(null);
    try {
      const result = await this.wifi.testarVelocidade();
      const current = this.diagnostics();
      if (current) {
        this.diagnostics.set({ ...current, speed: result });
      }
    } catch (erro) {
      this.speedError.set('Speed test failed');
      console.error('Speed test error:', erro);
    } finally {
      this.runningTest.set(false);
    }
  }
}
