import { describe, expect, it } from "vitest";

import { timestamp } from "./timestamp";

describe("timestamp", () => {
  it("accepts ISO-8601 UTC timestamps with or without milliseconds", () => {
    expect(timestamp("2026-10-13T18:30:05Z")).toBe("2026-10-13T18:30:05Z");
    expect(timestamp("2026-10-13T18:30:05.123Z")).toBe("2026-10-13T18:30:05.123Z");
    expect(timestamp(new Date(Date.UTC(2026, 9, 13, 18, 30, 5, 7)).toISOString())).toBe(
      "2026-10-13T18:30:05.007Z",
    );
  });

  it("accepts a leap day and end of day", () => {
    expect(timestamp("2024-02-29T23:59:59Z")).toBe("2024-02-29T23:59:59Z");
  });

  it("rejects non-UTC offsets, missing parts and impossible values", () => {
    expect(() => timestamp("2026-10-13T18:30:05-03:00")).toThrow(RangeError);
    expect(() => timestamp("2026-10-13T18:30:05")).toThrow(RangeError);
    expect(() => timestamp("2026-10-13")).toThrow(RangeError);
    expect(() => timestamp("2026-02-30T10:00:00Z")).toThrow(RangeError);
    expect(() => timestamp("2026-10-13T24:00:00Z")).toThrow(RangeError);
    expect(() => timestamp("2026-10-13T10:60:00Z")).toThrow(RangeError);
    expect(() => timestamp("2026-10-13T10:00:60Z")).toThrow(RangeError);
    expect(() => timestamp("")).toThrow(RangeError);
  });
});
