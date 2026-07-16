import { Routes } from "@angular/router";
import { Home } from "./pages/home/home";
import { Network } from "./pages/network/network";

export const routes: Routes = [
  { path: "", component: Home },
  { path: "network", component: Network },
];
