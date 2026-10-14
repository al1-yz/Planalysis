import { describe, expect, it } from "vitest";

import { isRepositoryErrorCode, RepositoryError } from "./errors";

describe("RepositoryError", () => {
  it("carries a machine-readable code and a message", () => {
    const error = new RepositoryError("duplicate_name", "Nome já em uso");
    expect(error).toBeInstanceOf(Error);
    expect(error.name).toBe("RepositoryError");
    expect(error.code).toBe("duplicate_name");
    expect(error.message).toBe("Nome já em uso");
  });

  it("recognizes the codes a repository can report", () => {
    expect(isRepositoryErrorCode("before_opening_date")).toBe(true);
    expect(isRepositoryErrorCode("in_use")).toBe(true);
    expect(isRepositoryErrorCode("not_a_code")).toBe(false);
    expect(isRepositoryErrorCode(42)).toBe(false);
  });
});
