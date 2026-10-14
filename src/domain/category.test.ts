import { describe, expect, it } from "vitest";

import { category, type CategoryInput } from "./category";
import { CREATED_AT, IDS, UPDATED_AT } from "./entities.fixtures";

const valid: CategoryInput = {
  id: IDS.category,
  name: " Alimentação ",
  kind: "expense",
  archivedAt: null,
  createdAt: CREATED_AT,
  updatedAt: UPDATED_AT,
};

describe("category", () => {
  it("builds a category with a normalized name", () => {
    expect(category(valid)).toEqual({ ...valid, name: "Alimentação" });
    expect(category({ ...valid, kind: "income" }).kind).toBe("income");
  });

  it("rejects invalid fields with a RangeError", () => {
    expect(() => category({ ...valid, id: "nope" })).toThrow(RangeError);
    expect(() => category({ ...valid, name: "" })).toThrow(RangeError);
    expect(() => category({ ...valid, kind: "transfer" })).toThrow(RangeError);
    expect(() => category({ ...valid, archivedAt: "x" })).toThrow(RangeError);
  });
});
