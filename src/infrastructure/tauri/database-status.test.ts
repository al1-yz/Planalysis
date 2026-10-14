import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";

import failedFixture from "../../../contract/database-status-failed.json";
import readyFixture from "../../../contract/database-status-ready.json";
import { RepositoryError } from "../../domain/errors";
import { getDatabaseStatus } from "./database-status";

afterEach(() => {
  clearMocks();
});

describe("getDatabaseStatus", () => {
  it("reads a ready database from the shared fixture", async () => {
    mockIPC((command) => (command === "database_status" ? readyFixture : null));
    await expect(getDatabaseStatus()).resolves.toEqual({
      ready: true,
      schemaVersion: 1,
      appVersion: "0.1.0",
      path: "/data/planalysis.db",
      error: null,
    });
  });

  it("reads why a database could not be opened", async () => {
    mockIPC(() => failedFixture);
    const status = await getDatabaseStatus();
    expect(status.ready).toBe(false);
    expect(status.error?.code).toBe("database_too_new");
  });

  it("rejects with a RepositoryError when the command itself fails", async () => {
    mockIPC(() => {
      throw "command database_status not found";
    });
    await expect(getDatabaseStatus()).rejects.toBeInstanceOf(RepositoryError);
  });
});
