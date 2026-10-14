import { describe, expect, it } from "vitest";

import { card, type CardInput } from "./card";
import { CREATED_AT, IDS, UPDATED_AT } from "./entities.fixtures";

const valid: CardInput = {
  id: IDS.card,
  name: "Nubank",
  closingDay: 10,
  dueDay: 17,
  creditLimit: 500000,
  archivedAt: null,
  createdAt: CREATED_AT,
  updatedAt: UPDATED_AT,
};

describe("card", () => {
  it("builds a card", () => {
    expect(card(valid)).toEqual(valid);
  });

  it("allows no credit limit, a zero limit, and days 1 and 31", () => {
    expect(card({ ...valid, creditLimit: null }).creditLimit).toBeNull();
    expect(card({ ...valid, creditLimit: 0 }).creditLimit).toBe(0);
    expect(card({ ...valid, closingDay: 1, dueDay: 31 })).toMatchObject({
      closingDay: 1,
      dueDay: 31,
    });
  });

  it("rejects invalid fields with a RangeError", () => {
    expect(() => card({ ...valid, id: "nope" })).toThrow(RangeError);
    expect(() => card({ ...valid, name: " " })).toThrow(RangeError);
    expect(() => card({ ...valid, closingDay: 0 })).toThrow(RangeError);
    expect(() => card({ ...valid, dueDay: 32 })).toThrow(RangeError);
    expect(() => card({ ...valid, closingDay: 10.5 })).toThrow(RangeError);
    expect(() => card({ ...valid, creditLimit: -1 })).toThrow(RangeError);
    expect(() => card({ ...valid, creditLimit: 1.5 })).toThrow(RangeError);
  });
});
