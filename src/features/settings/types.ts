export interface BackupInfo {
  fileName: string;
  path: string;
  kind: string;
  createdAt: number;
  appVersion: string | null;
  schemaVersion: number | null;
  sizeBytes: number;
}

export interface RestoreResult {
  sourcePath: string;
  preRestoreBackup: string;
  restartRequired: boolean;
}

export interface AppInfo {
  version: string;
  dataDir: string;
  databasePath: string;
  backupsDir: string;
  logsDir: string;
  schemaVersion: number;
}
