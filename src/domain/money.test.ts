import { describe, expect, it } from "vitest";

import { addCents, moneyCents, subtractCents, sumCents } from "./money";

describe("moneyCents", () => {
  it("accepts zero, positive and negative integer cents", () => {
    expect(moneyCents(1050)).toBe(1050);
    expect(moneyCents(0)).toBe(0);
    expect(moneyCents(-1050)).toBe(-1050);
  });

  it("rejects fractional, non-finite and unsafe values", () => {
    expect(() => moneyCents(10.5)).toThrow(RangeError);
    expect(() => moneyCents(Number.NaN)).toThrow(RangeError);
    expect(() => moneyCents(Number.POSITIVE_INFINITY)).toThrow(RangeError);
    expect(() => moneyCents(Number.MAX_SAFE_INTEGER + 1)).toThrow(RangeError);
    expect(() => moneyCents(Number.MIN_SAFE_INTEGER - 1)).toThrow(RangeError);
  });

  it("never produces negative zero", () => {
    expect(moneyCents(-0)).toBe(0);
  });
});

describe("money arithmetic", () => {
  it("adds, subtracts and sums exact cents", () => {
    expect(addCents(moneyCents(1050), moneyCents(250))).toBe(1300);
    expect(subtractCents(moneyCents(1050), moneyCents(1300))).toBe(-250);
    expect(sumCents([moneyCents(10), moneyCents(-3), moneyCents(5)])).toBe(12);
    expect(sumCents([])).toBe(0);
  });

  it("returns 0, not -0, when operands cancel out", () => {
    expect(addCents(moneyCents(-5), moneyCents(5))).toBe(0);
    expect(subtractCents(moneyCents(0), moneyCents(0))).toBe(0);
  });

  it("rejects results outside the safe integer range", () => {
    const max = moneyCents(Number.MAX_SAFE_INTEGER);
    const min = moneyCents(Number.MIN_SAFE_INTEGER);
    expect(() => addCents(max, moneyCents(1))).toThrow(RangeError);
    expect(() => subtractCents(min, moneyCents(1))).toThrow(RangeError);
    expect(() => sumCents([max, moneyCents(1)])).toThrow(RangeError);
  });

  it("detects overflow in a sum even when a later term would bring it back", () => {
    const max = moneyCents(Number.MAX_SAFE_INTEGER);
    expect(() => sumCents([max, moneyCents(1), moneyCents(-1)])).toThrow(RangeError);
  });
});
