import {
  addMonths,
  clampDayToMonth,
  dayOfMonth,
  toYearMonth,
  type LocalDate,
  type YearMonth,
} from "./dates";

/**
 * The invoice a purchase belongs to, identified by the `YearMonth` of its closing date.
 * Planalysis convention: a purchase before the closing date is on the current invoice; a
 * purchase on or after it is on the next one.
 */
export function assignInvoice(purchaseDate: LocalDate, closingDay: number): YearMonth {
  const purchaseMonth = toYearMonth(purchaseDate);
  return dayOfMonth(purchaseDate) < clampDayToMonth(purchaseMonth, closingDay)
    ? purchaseMonth
    : addMonths(purchaseMonth, 1);
}
