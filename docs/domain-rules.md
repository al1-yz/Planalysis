# Domain rules

Planalysis conventions for financial behavior. These are deterministic rules of this
application — not claims about how any bank behaves. Each rule must be covered by unit tests
when implemented.

## Money

- All BRL amounts are integer cents (`MoneyCents`, a lightweight branded `number`).
  `R$ 10,50` → `1050`. Values must be safe integers.
- Formatted strings are for display only, using
  `Intl.NumberFormat("pt-BR", { style: "currency", currency: "BRL" })`.
- Arithmetic stays in integers. Percentages that produce fractional cents are rounded half away
  from zero.

## Installment allocation

`sum(installments) === total`, always.

Rule: divide the total by the number of installments (integer division). The remainder cents
are added one each to the **first** installments.

Example: R$ 100,00 in 3 → R$ 33,34 · R$ 33,33 · R$ 33,33.

## Dates

| Concept                          | Representation                                  |
| -------------------------------- | ----------------------------------------------- |
| Calendar date (transaction etc.) | `LocalDate` string `YYYY-MM-DD`, no time/zone   |
| Accounting period                | `YearMonth` string `YYYY-MM`                    |
| Record timestamps                | ISO-8601 UTC string (`createdAt`, `updatedAt`)  |
| Day-of-month settings            | integer 1–31 (closing day, due day, recurrence) |

- `LocalDate` is never parsed with `new Date("YYYY-MM-DD")` (that is UTC and can shift the day).
- "Today" is taken from the device's local calendar date.
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

This is a fixed Planalysis convention, not a universal banking rule. It may become configurable
per card in the future; it is not configurable now.
