import { Injectable } from '@angular/core';
import { getInternetInfo, runSpeedTest } from '../../generated/commands';
import type { InternetDiagnostics, SpeedResult } from '../../generated/types';

@Injectable({ providedIn: 'root' })
export class WifiService {
  async carregar(): Promise<InternetDiagnostics> {
    return getInternetInfo();
  }

  async testarVelocidade(): Promise<SpeedResult> {
    return runSpeedTest();
  }
}
