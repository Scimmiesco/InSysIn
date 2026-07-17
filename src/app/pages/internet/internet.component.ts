import { AfterViewInit, Component, inject, OnInit } from "@angular/core";
import { CommonModule } from "@angular/common";
import { InternetStore } from "../../store/internet.store";
import { SectionComponent } from "../../components/section/section";
import { ButtonComponent } from "../../components/button/button";
import { InfoItemComponent } from "../../components/info-item/info-item";

@Component({
  selector: "app-internet",
  standalone: true,
  imports: [CommonModule, SectionComponent, ButtonComponent, InfoItemComponent],
  templateUrl: "./internet.component.html",
  styleUrl: "./internet.component.css",
})
export class Internet implements OnInit{
  protected store = inject(InternetStore);

  ngOnInit(): void {
    this.store.load();
  }
}
