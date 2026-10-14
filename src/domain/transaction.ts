import { localDate, type LocalDate } from "./dates";
import {
  accountId,
  cardId,
  categoryId,
  transactionId,
  transferId,
  type AccountId,
  type CardId,
  type CategoryId,
  type TransactionId,
  type TransferId,
} from "./ids";
import { moneyCents, type MoneyCents } from "./money";
import { timestamp, type Timestamp } from "./timestamp";

/**
 * A signed amount applied to one account or one card on the date the event happened: positive
 * is money in, negative is money out. Exactly one of `accountId` and `cardId` is set. A transfer
 * leg also has a `transferId` and never a category or a card.
 */
export interface Transaction {
  id: TransactionId;
  accountId: AccountId | null;
  cardId: CardId | null;
  categoryId: CategoryId | null;
  transferId: TransferId | null;
  date: LocalDate;
  amount: MoneyCents;
  description: string;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

/** Unvalidated transaction fields, as received from storage or a form. */
export interface TransactionInput {
  id: string;
  accountId: string | null;
  cardId: string | null;
  categoryId: string | null;
  transferId: string | null;
  date: string;
  amount: number;
  description: string;
  createdAt: string;
  updatedAt: string;
}

/** Validates raw fields into a {@link Transaction}. Throws `RangeError` on any invalid field. */
export function transaction(input: TransactionInput): Transaction {
  if ((input.accountId === null) === (input.cardId === null)) {
    throw new RangeError("A transaction belongs to exactly one account or one card.");
  }
  if (input.transferId !== null && (input.accountId === null || input.categoryId !== null)) {
    throw new RangeError("A transfer leg belongs to an account and has no category.");
  }
  const amount = moneyCents(input.amount);
  if (amount === 0) {
    throw new RangeError("A transaction amount cannot be zero.");
  }
  return {
    id: transactionId(input.id),
    accountId: input.accountId === null ? null : accountId(input.accountId),
    cardId: input.cardId === null ? null : cardId(input.cardId),
    categoryId: input.categoryId === null ? null : categoryId(input.categoryId),
    transferId: input.transferId === null ? null : transferId(input.transferId),
    date: localDate(input.date),
    amount,
    description: input.description.trim(),
    createdAt: timestamp(input.createdAt),
    updatedAt: timestamp(input.updatedAt),
  };
}
