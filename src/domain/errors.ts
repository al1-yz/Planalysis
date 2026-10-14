export const REPOSITORY_ERROR_CODES = [
  "not_found",
  "duplicate_name",
  "in_use",
  "before_opening_date",
  "opening_date_after_transactions",
  "archived_target",
  "invalid_transfer",
  "transfer_leg",
  "invalid_data",
  "database_unavailable",
  "internal",
] as const;

export type RepositoryErrorCode = (typeof REPOSITORY_ERROR_CODES)[number];

export function isRepositoryErrorCode(value: unknown): value is RepositoryErrorCode {
  return REPOSITORY_ERROR_CODES.some((code) => code === value);
}

/** A failure reported by persistence. Features translate `code` into user-facing pt-BR text. */
export class RepositoryError extends Error {
  readonly code: RepositoryErrorCode;

  constructor(code: RepositoryErrorCode, message: string) {
    super(message);
    this.name = "RepositoryError";
    this.code = code;
  }
}
