import { invoke } from "@tauri-apps/api/core";

import { isRepositoryErrorCode, RepositoryError } from "../../domain/errors";

/** The `{ code, message }` object the Rust side rejects with. */
function isWireError(value: unknown): value is { code: unknown; message: string } {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    typeof value.message === "string"
  );
}

function toRepositoryError(error: unknown): RepositoryError {
  if (isWireError(error)) {
    const code = isRepositoryErrorCode(error.code) ? error.code : "internal";
    return new RepositoryError(code, error.message);
  }
  return new RepositoryError("internal", error instanceof Error ? error.message : String(error));
}

/** Calls a Rust command. Every failure rejects with a {@link RepositoryError}. */
export async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toRepositoryError(error);
  }
}
