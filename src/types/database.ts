export interface DatabaseStatus {
  path: string;
  journalMode: string;
  foreignKeys: boolean;
  migrationCount: number;
  sizeBytes: number;
}
