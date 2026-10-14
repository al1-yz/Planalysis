import { dayOfMonthSetting } from "./dates";
import { cardId, type CardId } from "./ids";
import { entityName } from "./names";
import { moneyCents, type MoneyCents } from "./money";
import { timestamp, type Timestamp } from "./timestamp";

/** A credit card. Not an account: purchases on it are assigned to invoices by closing date. */
export interface Card {
  id: CardId;
  name: string;
  /** Day of month the invoice closes (1–31, clamped in short months). */
  closingDay: number;
  /** Day of month the invoice is due (1–31, clamped in short months). */
  dueDay: number;
  creditLimit: MoneyCents | null;
  archivedAt: Timestamp | null;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

/** Unvalidated card fields, as received from storage or a form. */
export interface CardInput {
  id: string;
  name: string;
  closingDay: number;
  dueDay: number;
  creditLimit: number | null;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

/** Validates raw fields into a {@link Card}. Throws `RangeError` on any invalid field. */
export function card(input: CardInput): Card {
  const creditLimit = input.creditLimit === null ? null : moneyCents(input.creditLimit);
  if (creditLimit !== null && creditLimit < 0) {
    throw new RangeError(`A credit limit cannot be negative, received ${creditLimit}.`);
  }
  return {
    id: cardId(input.id),
    name: entityName(input.name),
    closingDay: dayOfMonthSetting(input.closingDay),
    dueDay: dayOfMonthSetting(input.dueDay),
    creditLimit,
    archivedAt: input.archivedAt === null ? null : timestamp(input.archivedAt),
    createdAt: timestamp(input.createdAt),
    updatedAt: timestamp(input.updatedAt),
  };
}
