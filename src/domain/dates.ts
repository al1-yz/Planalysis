// Calendar arithmetic only: no `Date` objects, so results never depend on the device time zone.

/** A calendar date as `YYYY-MM-DD`, with no time or time zone. Create with {@link localDate}. */
export type LocalDate = string & { readonly __brand: "LocalDate" };

/** An accounting period as `YYYY-MM`. Create with {@link yearMonth}. */
export type YearMonth = string & { readonly __brand: "YearMonth" };

const LOCAL_DATE_PATTERN = /^(\d{4})-(\d{2})-(\d{2})$/;
const YEAR_MONTH_PATTERN = /^(\d{4})-(\d{2})$/;

function isLeapYear(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
}

function lengthOfMonth(year: number, month: number): number {
  if (month === 2) return isLeapYear(year) ? 29 : 28;
  return month === 4 || month === 6 || month === 9 || month === 11 ? 30 : 31;
}

function isValidYearMonth(year: number, month: number): boolean {
  return year >= 1 && month >= 1 && month <= 12;
}

/** Validates a strict `YYYY-MM-DD` real calendar date (year 0001–9999). Throws `RangeError`. */
export function localDate(value: string): LocalDate {
  const match = LOCAL_DATE_PATTERN.exec(value);
  const year = Number(match?.[1]);
  const month = Number(match?.[2]);
  const day = Number(match?.[3]);
  if (!match || !isValidYearMonth(year, month) || day < 1 || day > lengthOfMonth(year, month)) {
    throw new RangeError(`Invalid LocalDate "${value}": expected a real date as YYYY-MM-DD.`);
  }
  return value as LocalDate;
}

/** Validates a strict `YYYY-MM` period (year 0001–9999, month 01–12). Throws `RangeError`. */
export function yearMonth(value: string): YearMonth {
  const match = YEAR_MONTH_PATTERN.exec(value);
  if (!match || !isValidYearMonth(Number(match[1]), Number(match[2]))) {
    throw new RangeError(`Invalid YearMonth "${value}": expected YYYY-MM.`);
  }
  return value as YearMonth;
}

function splitYearMonth(value: YearMonth): { year: number; month: number } {
  return { year: Number(value.slice(0, 4)), month: Number(value.slice(5, 7)) };
}

function formatYearMonth(year: number, month: number): YearMonth {
  return `${String(year).padStart(4, "0")}-${String(month).padStart(2, "0")}` as YearMonth;
}

/** Number of days in the given month, accounting for leap years. */
export function daysInMonth(period: YearMonth): number {
  const { year, month } = splitYearMonth(period);
  return lengthOfMonth(year, month);
}

/**
 * Applies a day-of-month setting (1–31, e.g. a closing or due day) to a month, clamping to the
 * month's last day when it does not exist (31 in April → 30). Throws `RangeError` for a setting
 * outside 1–31.
 */
export function clampDayToMonth(period: YearMonth, day: number): number {
  if (!Number.isInteger(day) || day < 1 || day > 31) {
    throw new RangeError(`Day of month must be an integer from 1 to 31, received ${day}.`);
  }
  return Math.min(day, daysInMonth(period));
}

/** The accounting period that contains the given date. */
export function toYearMonth(date: LocalDate): YearMonth {
  return date.slice(0, 7) as YearMonth;
}

/** The day of the month (1–31) of a date. */
export function dayOfMonth(date: LocalDate): number {
  return Number(date.slice(8, 10));
}

/** Shifts a period by whole months (negative goes back). Throws if the result leaves 0001–9999. */
export function addMonths(period: YearMonth, months: number): YearMonth {
  if (!Number.isSafeInteger(months)) {
    throw new RangeError(`Months offset must be a safe integer, received ${months}.`);
  }
  const { year, month } = splitYearMonth(period);
  const index = year * 12 + (month - 1) + months;
  const targetYear = Math.floor(index / 12);
  if (targetYear < 1 || targetYear > 9999) {
    throw new RangeError(`Shifting ${period} by ${months} months leaves the supported years.`);
  }
  return formatYearMonth(targetYear, (((index % 12) + 12) % 12) + 1);
}
