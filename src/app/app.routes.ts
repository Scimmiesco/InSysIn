import { Routes } from "@angular/router";
import { Home } from "./pages/home/home";
import { Network } from "./pages/network/network.component";
import { Internet } from "./pages/internet/internet.component";

export const routes: Routes = [
  { path: "", component: Home },
  { path: "network", component: Network },
  { path: "internet", component: Internet },
];