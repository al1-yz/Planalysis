import { describe, expect, it } from "vitest";

import { assignInvoice } from "./card-invoice";
import { localDate } from "./dates";

describe("assignInvoice", () => {
  it("puts a purchase before the closing date on the current invoice", () => {
    expect(assignInvoice(localDate("2026-03-05"), 10)).toBe("2026-03");
    expect(assignInvoice(localDate("2026-03-09"), 10)).toBe("2026-03");
    expect(assignInvoice(localDate("2026-03-01"), 10)).toBe("2026-03");
  });

  it("puts a purchase on the closing date on the next invoice", () => {
    expect(assignInvoice(localDate("2026-03-10"), 10)).toBe("2026-04");
  });

  it("puts a purchase after the closing date on the next invoice", () => {
    expect(assignInvoice(localDate("2026-03-11"), 10)).toBe("2026-04");
    expect(assignInvoice(localDate("2026-03-31"), 10)).toBe("2026-04");
  });

  it("clamps a closing day that does not exist in the purchase month", () => {
    // Closing day 31 closes on 30 April: 29 April is before it, 30 April is on it.
    expect(assignInvoice(localDate("2026-04-29"), 31)).toBe("2026-04");
    expect(assignInvoice(localDate("2026-04-30"), 31)).toBe("2026-05");
    // February closes on the 28th (common year) or the 29th (leap year).
    expect(assignInvoice(localDate("2026-02-27"), 31)).toBe("2026-02");
    expect(assignInvoice(localDate("2026-02-28"), 31)).toBe("2026-03");
    expect(assignInvoice(localDate("2024-02-28"), 31)).toBe("2024-02");
    expect(assignInvoice(localDate("2024-02-29"), 31)).toBe("2024-03");
  });

  it("keeps a closing day of 31 on the 31st in long months", () => {
    expect(assignInvoice(localDate("2026-03-30"), 31)).toBe("2026-03");
    expect(assignInvoice(localDate("2026-03-31"), 31)).toBe("2026-04");
  });

  it("rolls into the next year when December purchases close into January", () => {
    expect(assignInvoice(localDate("2026-12-09"), 10)).toBe("2026-12");
    expect(assignInvoice(localDate("2026-12-10"), 10)).toBe("2027-01");
    expect(assignInvoice(localDate("2026-12-31"), 31)).toBe("2027-01");
  });

  it("handles the first possible closing day", () => {
    expect(assignInvoice(localDate("2026-03-01"), 1)).toBe("2026-04");
    expect(assignInvoice(localDate("2026-03-31"), 1)).toBe("2026-04");
  });

  it("rejects a closing day outside 1–31", () => {
    expect(() => assignInvoice(localDate("2026-03-05"), 0)).toThrow(RangeError);
    expect(() => assignInvoice(localDate("2026-03-05"), 32)).toThrow(RangeError);
    expect(() => assignInvoice(localDate("2026-03-05"), 10.5)).toThrow(RangeError);
  });

  it("rejects an invoice that would fall beyond year 9999", () => {
    expect(() => assignInvoice(localDate("9999-12-31"), 31)).toThrow(RangeError);
  });
});
