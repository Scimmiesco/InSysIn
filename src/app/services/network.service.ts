import { Injectable } from '@angular/core';
import { lerRede } from '../../generated/commands';
import { NetworkDashboard } from '../../generated/types';

@Injectable({ providedIn: 'root' })
export class NetworkService {
  async carregar(): Promise<NetworkDashboard> {
    return lerRede();
  }
}
