import { localDate, type LocalDate } from "./dates";
import { accountId, type AccountId } from "./ids";
import { entityName } from "./names";
import { moneyCents, type MoneyCents } from "./money";
import { timestamp, type Timestamp } from "./timestamp";

export const ACCOUNT_TYPES = ["checking", "savings", "cash"] as const;
export type AccountType = (typeof ACCOUNT_TYPES)[number];

/** A place the user holds money. Its balance is derived from the opening balance and transactions. */
export interface Account {
  id: AccountId;
  name: string;
  type: AccountType;
  /** The balance at the start of `openingDate`, before that day's transactions. */
  openingBalance: MoneyCents;
  openingDate: LocalDate;
  /** Set when the account is hidden from new use; its history is kept. */
  archivedAt: Timestamp | null;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

/** Unvalidated account fields, as received from storage or a form. */
export interface AccountInput {
  id: string;
  name: string;
  type: string;
  openingBalance: number;
  openingDate: string;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

/** Validates raw fields into an {@link Account}. Throws `RangeError` on any invalid field. */
export function account(input: AccountInput): Account {
  const type = ACCOUNT_TYPES.find((candidate) => candidate === input.type);
  if (type === undefined) {
    throw new RangeError(`Invalid account type "${input.type}".`);
  }
  return {
    id: accountId(input.id),
    name: entityName(input.name),
    type,
    openingBalance: moneyCents(input.openingBalance),
    openingDate: localDate(input.openingDate),
    archivedAt: input.archivedAt === null ? null : timestamp(input.archivedAt),
    createdAt: timestamp(input.createdAt),
    updatedAt: timestamp(input.updatedAt),
  };
}
