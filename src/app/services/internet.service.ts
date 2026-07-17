import { Injectable } from '@angular/core';
import { getInternetInfo, getLocalNetworkInfo, runSpeedTest } from '../../generated/commands';
import type { InternetDiagnostics, LocalNetworkInfo, SpeedResult } from '../../generated/types';

@Injectable({ providedIn: 'root' })
export class InternetService {
  async carregar(): Promise<InternetDiagnostics> {
    return getInternetInfo();
  }

  async testarVelocidade(): Promise<SpeedResult> {
    return runSpeedTest();
  }

  async carregarRedeLocal(): Promise<LocalNetworkInfo> {
    return getLocalNetworkInfo();
  }
}
