import { invokeCommand } from "@/lib/ipc";
import type { DatabaseStatus } from "@/types/database";

import type { AppInfo, BackupInfo, RestoreResult } from "./types";

export function getDatabaseStatus(): Promise<DatabaseStatus> {
  return invokeCommand<DatabaseStatus>("get_database_status");
}

export function listBackups(): Promise<BackupInfo[]> {
  return invokeCommand<BackupInfo[]>("list_backups");
}

export function createBackup(destination: string | null): Promise<BackupInfo> {
  return invokeCommand<BackupInfo>("create_backup", { destination });
}

export function stageRestore(sourcePath: string): Promise<RestoreResult> {
  return invokeCommand<RestoreResult>("stage_restore", { sourcePath });
}

export function cancelPendingRestore(): Promise<boolean> {
  return invokeCommand<boolean>("cancel_pending_restore");
}

export function getAppInfo(): Promise<AppInfo> {
  return invokeCommand<AppInfo>("get_app_info");
}

export function restartApp(): Promise<void> {
  return invokeCommand<void>("restart_app");
}
