import { Routes } from "@angular/router";
import { Home } from "./pages/home/home";
import { Network } from "./pages/network/network";
import { Wifi } from "./pages/wifi/wifi";

export const routes: Routes = [
  { path: "", component: Home },
  { path: "network", component: Network },
  { path: "wifi", component: Wifi },
];
