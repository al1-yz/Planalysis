# Domain rules

Planalysis conventions for financial behavior. These are deterministic rules of this
application — not claims about how any bank behaves. Each rule must be covered by unit tests
when implemented.

## Money

- All BRL amounts are integer cents (`MoneyCents`, a lightweight branded `number`).
  `R$ 10,50` → `1050`. Values must be safe integers, and so must the result of every addition
  or subtraction; an out-of-range result is an error, never a silently imprecise number. Zero and
  negative amounts (e.g. refunds) are valid.
- Formatted strings are for display only, using
  `Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })`. Formatting lives in the UI
  layer (`src/features`, `src/components`), never in `src/domain`.
- Arithmetic stays in integers.
- Percentage calculations are not implemented yet. When a feature needs them, the convention is
  that percentages producing fractional cents are rounded half away from zero.

## Installment allocation

`sum(installments) === total`, always.

Rule: divide the total by the number of installments (integer division). The remainder cents
are added one each to the **first** installments.

Example: R$ 100,00 in 3 → R$ 33,34 · R$ 33,33 · R$ 33,33.

A negative total (refund) is split the same way in magnitude: -R$ 100,00 in 3 → -R$ 33,34 ·
-R$ 33,33 · -R$ 33,33. A total smaller than the count gives one cent each to the first
installments and zero to the rest. The count must be a positive integer.

There is no maximum count yet. One must be set before installment counts become user-controlled.

## Dates

| Concept                          | Representation                                  |
| -------------------------------- | ----------------------------------------------- |
| Calendar date (transaction etc.) | `LocalDate` string `YYYY-MM-DD`, no time/zone   |
| Accounting period                | `YearMonth` string `YYYY-MM`                    |
| Record timestamps                | ISO-8601 UTC string (`createdAt`, `updatedAt`)  |
| Day-of-month settings            | integer 1–31 (closing day, due day, recurrence) |

- `LocalDate` is never parsed with `new Date("YYYY-MM-DD")` (that is UTC and can shift the day).
- Pure date calculations (validation, clamping, month arithmetic, invoice assignment) are calendar
  arithmetic and never depend on the device time zone.
- Years are four digits, `0001` to `9999`. Invalid values (`2026-02-30`, month `13`, non-padded
  strings) are rejected with a `RangeError`, and so is any operation whose result falls outside
  those years (e.g. `addMonths` past `9999-12`, or an invoice after `9999-12`).
- "Today" is taken from the device's local calendar date. This is the only device input and is
  not implemented yet; it will be added with the first feature that needs it.
- A day-of-month that does not exist in a given month is clamped to that month's last day
  (closing day 31 → 30 April, 28 or 29 February).

## Transfers

A transfer between two owned accounts is never counted as income or expense. It only moves
balance between accounts.

## Credit card invoice assignment — Planalysis convention

Given a card's closing day (clamped per month as above):

| Purchase date           | Invoice         |
| ----------------------- | --------------- |
| before the closing date | current invoice |
| **on** the closing date | next invoice    |
| after the closing date  | next invoice    |

The invoice is identified by the `YearMonth` of its closing date: with closing day 10, a
purchase on 2026-03-05 is on invoice `2026-03` and one on 2026-03-10 is on `2026-04`.

This is a fixed Planalysis convention, not a universal banking rule. It may become configurable
per card in the future; it is not configurable now.
