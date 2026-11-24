import { invokeCommand } from "@/lib/ipc";

import type { DashboardData } from "./types";

export function getDashboard(): Promise<DashboardData> {
  return invokeCommand<DashboardData>("get_dashboard");
}
