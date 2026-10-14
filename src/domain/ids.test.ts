import { describe, expect, it } from "vitest";

import { accountId, cardId, categoryId, transactionId, transferId } from "./ids";

const VALID = "3f2b8c1e-5d4a-4e6f-9a1b-0c7d8e9f1a2b";

describe("entity identifiers", () => {
  const constructors = { accountId, cardId, categoryId, transactionId, transferId };

  for (const [name, create] of Object.entries(constructors)) {
    it(`${name} accepts a canonical lowercase UUID`, () => {
      expect(create(VALID)).toBe(VALID);
    });

    it(`${name} rejects anything that is not a canonical lowercase UUID`, () => {
      expect(() => create("")).toThrow(RangeError);
      expect(() => create("not-a-uuid")).toThrow(RangeError);
      expect(() => create(VALID.toUpperCase())).toThrow(RangeError);
      expect(() => create(` ${VALID}`)).toThrow(RangeError);
      expect(() => create(VALID.replaceAll("-", ""))).toThrow(RangeError);
      expect(() => create("3f2b8c1e-5d4a-4e6f-9a1b-0c7d8e9f1a2g")).toThrow(RangeError);
    });
  }
});
