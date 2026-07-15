import { Component } from '@angular/core';
import { DecimalPipe, DatePipe, CommonModule } from '@angular/common';
import { MemoryChart } from '../../components/memory-chart/memory-chart';
import { SystemStore } from '../../store/system.store';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [DecimalPipe, DatePipe, CommonModule, MemoryChart],
  templateUrl: './home.html',
  styleUrl: './home.css',
})
export class Home {
  constructor(protected store: SystemStore) {}
}
