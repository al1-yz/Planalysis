import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";

import { RepositoryError } from "../../domain/errors";
import { call } from "./invoke";

afterEach(() => {
  clearMocks();
});

async function failure(promise: Promise<unknown>): Promise<RepositoryError> {
  try {
    await promise;
  } catch (error) {
    if (error instanceof RepositoryError) {
      return error;
    }
    throw error;
  }
  throw new Error("expected the call to be rejected");
}

describe("call", () => {
  it("returns the command's result and passes its arguments", async () => {
    mockIPC((command, args) => ({ command, args }));
    await expect(call("account_get", { id: "x" })).resolves.toEqual({
      command: "account_get",
      args: { id: "x" },
    });
  });

  it("turns a structured Rust error into a RepositoryError with its code", async () => {
    mockIPC(() => {
      throw { code: "duplicate_name", message: "UNIQUE constraint failed" };
    });
    const error = await failure(call("account_create", {}));
    expect(error.code).toBe("duplicate_name");
    expect(error.message).toBe("UNIQUE constraint failed");
  });

  it("reports an unknown error code as an internal error", async () => {
    mockIPC(() => {
      throw { code: "something_new", message: "from a newer backend" };
    });
    const error = await failure(call("account_create", {}));
    expect(error.code).toBe("internal");
    expect(error.message).toBe("from a newer backend");
  });

  it("wraps plain-string and Error failures, such as rejected arguments", async () => {
    mockIPC(() => {
      throw "invalid args `account` for command `account_create`";
    });
    const fromString = await failure(call("account_create", {}));
    expect(fromString.code).toBe("internal");
    expect(fromString.message).toContain("invalid args");

    mockIPC(() => {
      throw new Error("boom");
    });
    const fromError = await failure(call("account_create", {}));
    expect(fromError).toMatchObject({ code: "internal", message: "boom" });
  });
});
