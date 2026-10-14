import { localDate, type LocalDate } from "./dates";
import { accountId, transferId, type AccountId, type TransactionId, type TransferId } from "./ids";
import { moneyCents, subtractCents, type MoneyCents } from "./money";
import { timestamp, type Timestamp } from "./timestamp";
import { transaction, type Transaction } from "./transaction";

/**
 * Money moved between two accounts the user owns. It is stored as two linked transactions (see
 * {@link transferToLegs}) and is never income or expense.
 */
export interface Transfer {
  id: TransferId;
  fromAccountId: AccountId;
  toAccountId: AccountId;
  /** Always positive: the amount that left `fromAccountId` and reached `toAccountId`. */
  amount: MoneyCents;
  date: LocalDate;
  description: string;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

/** Unvalidated transfer fields, as received from a form. */
export interface TransferInput {
  id: string;
  fromAccountId: string;
  toAccountId: string;
  amount: number;
  date: string;
  description: string;
  createdAt: string;
  updatedAt: string;
}

/** The two transactions of a transfer: outgoing (negative) first, incoming (positive) second. */
export type TransferLegs = readonly [Transaction, Transaction];

/** Validates raw fields into a {@link Transfer}. Throws `RangeError` on any invalid field. */
export function transfer(input: TransferInput): Transfer {
  const from = accountId(input.fromAccountId);
  const to = accountId(input.toAccountId);
  if (from === to) {
    throw new RangeError("A transfer needs two different accounts.");
  }
  const amount = moneyCents(input.amount);
  if (amount <= 0) {
    throw new RangeError(`A transfer amount must be positive, received ${amount}.`);
  }
  return {
    id: transferId(input.id),
    fromAccountId: from,
    toAccountId: to,
    amount,
    date: localDate(input.date),
    description: input.description.trim(),
    createdAt: timestamp(input.createdAt),
    updatedAt: timestamp(input.updatedAt),
  };
}

/** Splits a transfer into its outgoing and incoming legs, using the given leg ids in that order. */
export function transferToLegs(
  value: Transfer,
  legIds: readonly [TransactionId, TransactionId],
): TransferLegs {
  const common = {
    cardId: null,
    categoryId: null,
    transferId: value.id,
    date: value.date,
    description: value.description,
    createdAt: value.createdAt,
    updatedAt: value.updatedAt,
  };
  return [
    transaction({
      ...common,
      id: legIds[0],
      accountId: value.fromAccountId,
      amount: subtractCents(moneyCents(0), value.amount),
    }),
    transaction({ ...common, id: legIds[1], accountId: value.toAccountId, amount: value.amount }),
  ];
}

/** Rebuilds a transfer from its two legs, in any order. Throws `RangeError` unless they are a valid pair. */
export function transferFromLegs(legs: readonly Transaction[]): Transfer {
  const outgoing = legs.find((leg) => leg.amount < 0);
  const incoming = legs.find((leg) => leg.amount > 0);
  if (legs.length !== 2 || outgoing === undefined || incoming === undefined) {
    throw new RangeError("A transfer has exactly two legs: one outgoing and one incoming.");
  }
  const { transferId: linkedId, accountId: fromAccount } = outgoing;
  const toAccount = incoming.accountId;
  const sameEvent =
    outgoing.date === incoming.date &&
    outgoing.description === incoming.description &&
    outgoing.amount + incoming.amount === 0;
  const plainLegs =
    outgoing.cardId === null &&
    incoming.cardId === null &&
    outgoing.categoryId === null &&
    incoming.categoryId === null;
  if (
    linkedId === null ||
    linkedId !== incoming.transferId ||
    fromAccount === null ||
    toAccount === null ||
    !sameEvent ||
    !plainLegs
  ) {
    throw new RangeError("The legs do not form a valid transfer.");
  }
  return transfer({
    id: linkedId,
    fromAccountId: fromAccount,
    toAccountId: toAccount,
    amount: incoming.amount,
    date: outgoing.date,
    description: outgoing.description,
    createdAt: outgoing.createdAt,
    updatedAt: outgoing.updatedAt,
  });
}
