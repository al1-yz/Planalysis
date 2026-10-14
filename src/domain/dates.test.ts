import { describe, expect, it } from "vitest";

import {
  addMonths,
  clampDayToMonth,
  dayOfMonth,
  daysInMonth,
  localDate,
  toYearMonth,
  yearMonth,
} from "./dates";

describe("localDate", () => {
  it("accepts real calendar dates", () => {
    expect(localDate("2026-03-05")).toBe("2026-03-05");
    expect(localDate("0001-01-01")).toBe("0001-01-01");
    expect(localDate("9999-12-31")).toBe("9999-12-31");
  });

  it("accepts 29 February only in leap years", () => {
    expect(localDate("2024-02-29")).toBe("2024-02-29");
    expect(localDate("2000-02-29")).toBe("2000-02-29");
    expect(() => localDate("2023-02-29")).toThrow(RangeError);
    expect(() => localDate("1900-02-29")).toThrow(RangeError);
  });

  it("rejects days and months that do not exist", () => {
    expect(() => localDate("2026-02-30")).toThrow(RangeError);
    expect(() => localDate("2026-04-31")).toThrow(RangeError);
    expect(() => localDate("2026-13-01")).toThrow(RangeError);
    expect(() => localDate("2026-00-10")).toThrow(RangeError);
    expect(() => localDate("2026-01-00")).toThrow(RangeError);
  });

  it("rejects anything that is not strictly YYYY-MM-DD", () => {
    expect(() => localDate("2026-3-5")).toThrow(RangeError);
    expect(() => localDate("26-03-05")).toThrow(RangeError);
    expect(() => localDate("0000-01-01")).toThrow(RangeError);
    expect(() => localDate("2026-03-05T10:00:00Z")).toThrow(RangeError);
    expect(() => localDate(" 2026-03-05")).toThrow(RangeError);
    expect(() => localDate("05/03/2026")).toThrow(RangeError);
    expect(() => localDate("")).toThrow(RangeError);
  });
});

describe("yearMonth", () => {
  it("accepts YYYY-MM with a month from 01 to 12", () => {
    expect(yearMonth("2026-03")).toBe("2026-03");
    expect(yearMonth("0001-01")).toBe("0001-01");
    expect(yearMonth("9999-12")).toBe("9999-12");
  });

  it("rejects invalid months and malformed strings", () => {
    expect(() => yearMonth("2026-00")).toThrow(RangeError);
    expect(() => yearMonth("2026-13")).toThrow(RangeError);
    expect(() => yearMonth("0000-05")).toThrow(RangeError);
    expect(() => yearMonth("2026-3")).toThrow(RangeError);
    expect(() => yearMonth("2026-03-05")).toThrow(RangeError);
    expect(() => yearMonth("")).toThrow(RangeError);
  });
});

describe("daysInMonth", () => {
  it("returns the length of each month", () => {
    expect(daysInMonth(yearMonth("2026-01"))).toBe(31);
    expect(daysInMonth(yearMonth("2026-04"))).toBe(30);
    expect(daysInMonth(yearMonth("2026-06"))).toBe(30);
    expect(daysInMonth(yearMonth("2026-09"))).toBe(30);
    expect(daysInMonth(yearMonth("2026-11"))).toBe(30);
    expect(daysInMonth(yearMonth("2026-12"))).toBe(31);
  });

  it("handles February in common, leap and century years", () => {
    expect(daysInMonth(yearMonth("2023-02"))).toBe(28);
    expect(daysInMonth(yearMonth("2024-02"))).toBe(29);
    expect(daysInMonth(yearMonth("1900-02"))).toBe(28);
    expect(daysInMonth(yearMonth("2000-02"))).toBe(29);
  });
});

describe("clampDayToMonth", () => {
  it("keeps a day that exists in the month", () => {
    expect(clampDayToMonth(yearMonth("2026-03"), 31)).toBe(31);
    expect(clampDayToMonth(yearMonth("2026-03"), 1)).toBe(1);
  });

  it("clamps a day that does not exist to the last day of the month", () => {
    expect(clampDayToMonth(yearMonth("2026-04"), 31)).toBe(30);
    expect(clampDayToMonth(yearMonth("2026-02"), 30)).toBe(28);
    expect(clampDayToMonth(yearMonth("2024-02"), 31)).toBe(29);
  });

  it("rejects a day-of-month setting outside 1–31", () => {
    expect(() => clampDayToMonth(yearMonth("2026-03"), 0)).toThrow(RangeError);
    expect(() => clampDayToMonth(yearMonth("2026-03"), 32)).toThrow(RangeError);
    expect(() => clampDayToMonth(yearMonth("2026-03"), 10.5)).toThrow(RangeError);
  });
});

describe("toYearMonth", () => {
  it("extracts the period of a date", () => {
    expect(toYearMonth(localDate("2026-03-05"))).toBe("2026-03");
    expect(toYearMonth(localDate("2024-02-29"))).toBe("2024-02");
  });
});

describe("dayOfMonth", () => {
  it("extracts the day of a date", () => {
    expect(dayOfMonth(localDate("2026-03-05"))).toBe(5);
    expect(dayOfMonth(localDate("2026-03-31"))).toBe(31);
    expect(dayOfMonth(localDate("2024-02-29"))).toBe(29);
  });
});

describe("addMonths", () => {
  it("moves forward and backward within a year", () => {
    expect(addMonths(yearMonth("2026-03"), 1)).toBe("2026-04");
    expect(addMonths(yearMonth("2026-03"), -2)).toBe("2026-01");
    expect(addMonths(yearMonth("2026-03"), 0)).toBe("2026-03");
  });

  it("crosses year boundaries in both directions", () => {
    expect(addMonths(yearMonth("2026-12"), 1)).toBe("2027-01");
    expect(addMonths(yearMonth("2026-01"), -1)).toBe("2025-12");
    expect(addMonths(yearMonth("2026-11"), 14)).toBe("2028-01");
    expect(addMonths(yearMonth("2026-02"), -26)).toBe("2023-12");
  });

  it("rejects fractional offsets and results outside years 0001–9999", () => {
    expect(() => addMonths(yearMonth("2026-03"), 0.5)).toThrow(RangeError);
    expect(() => addMonths(yearMonth("9999-12"), 1)).toThrow(RangeError);
    expect(() => addMonths(yearMonth("0001-01"), -1)).toThrow(RangeError);
  });
});
