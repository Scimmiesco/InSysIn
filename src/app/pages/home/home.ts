import { DecimalPipe } from "@angular/common";
import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  inject,
  OnInit,
  signal,
} from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { lerHardware } from "../../../generated/commands";
import { SysStats } from "../../../generated/types";

@Component({
  selector: "app-home",
  imports: [DecimalPipe],
  templateUrl: "./home.html",
  styleUrl: "./home.css",
})
export class Home implements OnInit {
  sys_info = signal<SysStats | null>(null);
  counter = signal(0);

  ngOnInit(): void {
    this.lerDados();

    setInterval(() => {
      this.lerDados();
    }, 1000);
  }

  async lerDados() {
    lerHardware().then((res) => {
      this.sys_info.set(res);
    });
  }
}
