import { describe, expect, it } from "vitest";

import { allocateInstallments } from "./installments";
import { moneyCents, sumCents } from "./money";

describe("allocateInstallments", () => {
  it("gives the remainder cents, one each, to the first installments", () => {
    expect(allocateInstallments(moneyCents(10000), 3)).toEqual([3334, 3333, 3333]);
    expect(allocateInstallments(moneyCents(10), 4)).toEqual([3, 3, 2, 2]);
  });

  it("splits evenly when there is no remainder", () => {
    expect(allocateInstallments(moneyCents(9000), 3)).toEqual([3000, 3000, 3000]);
  });

  it("returns the whole amount for a single installment", () => {
    expect(allocateInstallments(moneyCents(12345), 1)).toEqual([12345]);
  });

  it("allocates a negative total with the extra cents, in magnitude, on the first installments", () => {
    expect(allocateInstallments(moneyCents(-10000), 3)).toEqual([-3334, -3333, -3333]);
    expect(allocateInstallments(moneyCents(-9000), 3)).toEqual([-3000, -3000, -3000]);
  });

  it("gives one cent to the first installments when the total is smaller than the count", () => {
    expect(allocateInstallments(moneyCents(2), 5)).toEqual([1, 1, 0, 0, 0]);
    expect(allocateInstallments(moneyCents(-2), 5)).toEqual([-1, -1, 0, 0, 0]);
  });

  it("allocates a zero total as zero installments without negative zero", () => {
    expect(allocateInstallments(moneyCents(0), 3)).toEqual([0, 0, 0]);
  });

  it("rejects a count that is not a positive integer", () => {
    expect(() => allocateInstallments(moneyCents(100), 0)).toThrow(RangeError);
    expect(() => allocateInstallments(moneyCents(100), -3)).toThrow(RangeError);
    expect(() => allocateInstallments(moneyCents(100), 2.5)).toThrow(RangeError);
    expect(() => allocateInstallments(moneyCents(100), Number.NaN)).toThrow(RangeError);
  });

  it("always sums back to the total and never differs by more than one cent", () => {
    const totals = [
      0,
      1,
      7,
      99,
      10000,
      123457,
      -1,
      -7,
      -10000,
      -123457,
      Number.MAX_SAFE_INTEGER,
      Number.MIN_SAFE_INTEGER,
    ];
    const cases = totals.flatMap((raw) =>
      Array.from({ length: 24 }, (_, index) => ({ raw, count: index + 1 })),
    );
    for (const { raw, count } of cases) {
      const parts = allocateInstallments(moneyCents(raw), count);
      const context = `total=${raw} count=${count}`;
      expect(parts, context).toHaveLength(count);
      expect(sumCents(parts), context).toBe(raw);
      expect(Math.max(...parts) - Math.min(...parts), context).toBeLessThanOrEqual(1);
    }
  });
});
