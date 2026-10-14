import { localDate } from "./dates";

/** An instant as an ISO-8601 UTC string, e.g. `2026-10-13T18:30:05.123Z`. Create with {@link timestamp}. */
export type Timestamp = string & { readonly __brand: "Timestamp" };

const TIMESTAMP_PATTERN = /^(\d{4}-\d{2}-\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d{1,3})?Z$/;

/** Validates an ISO-8601 UTC timestamp (`...Z`, optional milliseconds). Throws `RangeError`. */
export function timestamp(value: string): Timestamp {
  const match = TIMESTAMP_PATTERN.exec(value);
  if (match) {
    const [, date, hour, minute, second] = match;
    try {
      localDate(date ?? "");
      if (Number(hour) < 24 && Number(minute) < 60 && Number(second) < 60) {
        return value as Timestamp;
      }
    } catch {
      // fall through to the shared error below
    }
  }
  throw new RangeError(
    `Invalid Timestamp "${value}": expected ISO-8601 UTC, e.g. 2026-10-13T18:30:05Z.`,
  );
}
