import { call } from "./invoke";

export const STARTUP_ERROR_CODES = [
  "database_too_new",
  "not_a_planalysis_database",
  "migration_failed",
  "database_unavailable",
] as const;

export type StartupErrorCode = (typeof STARTUP_ERROR_CODES)[number];

/** Whether the database opened at startup and, if not, why. */
export interface DatabaseStatus {
  ready: boolean;
  schemaVersion: number | null;
  appVersion: string;
  /** Where the database file lives (or would live). */
  path: string;
  error: { code: StartupErrorCode; message: string } | null;
}

export function getDatabaseStatus(): Promise<DatabaseStatus> {
  return call<DatabaseStatus>("database_status");
}
