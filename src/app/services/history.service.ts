import { Injectable } from '@angular/core';
import { obterHistorico, obterProcessosAgrupados } from '../../generated/commands';
import { HistoricoCompleto, ProcessoAgrupado } from '../../generated/types';

@Injectable({ providedIn: 'root' })
export class HistoryService {
  async carregar(): Promise<HistoricoCompleto> {
    return obterHistorico();
  }

  async carregarAgrupados(ordem: string, desc: boolean): Promise<ProcessoAgrupado[]> {
    return obterProcessosAgrupados(ordem, desc);
  }
}
