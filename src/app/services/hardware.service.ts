import { Injectable } from '@angular/core';
import { lerHardware } from '../../generated/commands';
import { SysStats } from '../../generated/types';

@Injectable({ providedIn: 'root' })
export class HardwareService {
  async carregar(): Promise<SysStats> {
    return lerHardware();
  }
}
