import { invokeCommand } from "@/lib/ipc";
import type { DatabaseStatus } from "@/types/database";

export function getDatabaseStatus(): Promise<DatabaseStatus> {
  return invokeCommand<DatabaseStatus>("get_database_status");
}
