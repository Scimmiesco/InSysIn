import { Component } from "@angular/core";
import { RouterOutlet, Router, RouterLink, RouterLinkActive } from "@angular/router";
import { listen } from "@tauri-apps/api/event";
import { ThemeService } from "./services/theme.service";

@Component({
  selector: "app-root",
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent {
  constructor(_theme: ThemeService, router: Router) {
    listen<string>("navigate", (e) => router.navigate([e.payload]));
  }
}
