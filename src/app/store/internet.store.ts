import { inject, Injectable, signal } from "@angular/core";
import type { InternetDiagnostics, LocalNetworkInfo } from "../../generated/types";
import { InternetService } from "../services/internet.service";

@Injectable({ providedIn: "root" })
export class InternetStore {

  private  internetService = inject(InternetService);
  diagnostics = signal<InternetDiagnostics | null>(null);
  localNetwork = signal<LocalNetworkInfo | null>(null);
  error = signal<string | null>(null);
  scanning = signal(true);
  runningTest = signal(false);
  speedError = signal<string | null>(null);
  scanningLocal = signal(true);

  constructor() {}

  load(): void {
    this.fetchInfo();
    this.fetchLocalNetwork();
  }

  private async fetchInfo(): Promise<void> {
    this.scanning.set(true);
    this.error.set(null);
    try {
      const res = await this.internetService.carregar();
      this.diagnostics.set(res);
    } catch (erro) {
      this.error.set("Failed to fetch internet info");
      console.error("Internet diagnostics error:", erro);
    } finally {
      this.scanning.set(false);
    }
  }

  refreshAll(): void {
    this.fetchInfo();
    this.fetchLocalNetwork();
  }

  async runSpeedTest(): Promise<void> {
    if (this.runningTest()) return;
    this.runningTest.set(true);
    this.speedError.set(null);
    try {
      const result = await this.internetService.testarVelocidade();
      const current = this.diagnostics();
      if (current) {
        this.diagnostics.set({ ...current, speed: result });
      }
    } catch (erro) {
      this.speedError.set("Speed test failed");
      console.error("Speed test error:", erro);
    } finally {
      this.runningTest.set(false);
    }
  }

  private async fetchLocalNetwork(): Promise<void> {
    this.scanningLocal.set(true);
    try {
      const res = await this.internetService.carregarRedeLocal();
      this.localNetwork.set(res);
    } catch (erro) {
      console.error("Local network error:", erro);
    } finally {
      this.scanningLocal.set(false);
    }
  }

  scanLocal(): void {
    this.fetchLocalNetwork();
  }
}
