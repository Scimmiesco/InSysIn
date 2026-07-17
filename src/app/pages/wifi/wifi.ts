import { Component, OnInit } from "@angular/core";
import { CommonModule } from "@angular/common";
import { WifiStore } from "../../store/wifi.store";

@Component({
  selector: "app-wifi",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./wifi.html",
  styleUrl: "./wifi.css",
})
export class Wifi implements OnInit {
  constructor(protected store: WifiStore) {}

  ngOnInit(): void {
    this.store.fetchInfo();
  }
}
