import { describe, expect, it } from "vitest";

import { account, type AccountInput } from "./account";
import { CREATED_AT, IDS, UPDATED_AT } from "./entities.fixtures";

const valid: AccountInput = {
  id: IDS.account,
  name: "  Conta   Corrente ",
  type: "checking",
  openingBalance: 150075,
  openingDate: "2026-01-15",
  archivedAt: null,
  createdAt: CREATED_AT,
  updatedAt: UPDATED_AT,
};

describe("account", () => {
  it("builds an account with a normalized name and typed fields", () => {
    expect(account(valid)).toEqual({ ...valid, name: "Conta Corrente" });
  });

  it("accepts every account type, a negative opening balance and an archive timestamp", () => {
    for (const type of ["checking", "savings", "cash"]) {
      expect(account({ ...valid, type }).type).toBe(type);
    }
    expect(account({ ...valid, openingBalance: -5000 }).openingBalance).toBe(-5000);
    expect(account({ ...valid, archivedAt: UPDATED_AT }).archivedAt).toBe(UPDATED_AT);
  });

  it("rejects invalid fields with a RangeError", () => {
    expect(() => account({ ...valid, id: "nope" })).toThrow(RangeError);
    expect(() => account({ ...valid, name: "  " })).toThrow(RangeError);
    expect(() => account({ ...valid, type: "investment" })).toThrow(RangeError);
    expect(() => account({ ...valid, openingBalance: 10.5 })).toThrow(RangeError);
    expect(() => account({ ...valid, openingDate: "2026-02-30" })).toThrow(RangeError);
    expect(() => account({ ...valid, archivedAt: "yesterday" })).toThrow(RangeError);
    expect(() => account({ ...valid, createdAt: "2026-10-13" })).toThrow(RangeError);
    expect(() => account({ ...valid, updatedAt: "" })).toThrow(RangeError);
  });
});
