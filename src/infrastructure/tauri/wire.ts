// The JSON shapes exchanged with the Rust commands (see `contract/` and `src-tauri/src/dto.rs`).
// Entities cross the boundary field for field; accounts, categories and cards also carry the
// normalized `nameKey` that the database's unique-name indexes are built on.

import { account, type Account, type AccountInput } from "../../domain/account";
import { card, type Card, type CardInput } from "../../domain/card";
import { category, type Category, type CategoryInput } from "../../domain/category";
import { RepositoryError } from "../../domain/errors";
import { nameKey } from "../../domain/names";
import type { TransactionFilter } from "../../domain/repositories";
import { transaction, type Transaction, type TransactionInput } from "../../domain/transaction";
import type { TransferLegs } from "../../domain/transfer";

export interface AccountWire extends AccountInput {
  nameKey: string;
}
export interface CategoryWire extends CategoryInput {
  nameKey: string;
}
export interface CardWire extends CardInput {
  nameKey: string;
}
export type TransactionWire = TransactionInput;

export interface TransactionFilterWire {
  accountId: string | null;
  cardId: string | null;
  categoryId: string | null;
  from: string | null;
  to: string | null;
}

export function accountToWire(value: Account): AccountWire {
  return { ...value, nameKey: nameKey(value.name) };
}

export function accountFromWire(wire: AccountWire): Account {
  return account(wire);
}

export function categoryToWire(value: Category): CategoryWire {
  return { ...value, nameKey: nameKey(value.name) };
}

export function categoryFromWire(wire: CategoryWire): Category {
  return category(wire);
}

export function cardToWire(value: Card): CardWire {
  return { ...value, nameKey: nameKey(value.name) };
}

export function cardFromWire(wire: CardWire): Card {
  return card(wire);
}

export function transactionToWire(value: Transaction): TransactionWire {
  return { ...value };
}

export function transactionFromWire(wire: TransactionWire): Transaction {
  return transaction(wire);
}

export function filterToWire(filter: TransactionFilter = {}): TransactionFilterWire {
  return {
    accountId: filter.accountId ?? null,
    cardId: filter.cardId ?? null,
    categoryId: filter.categoryId ?? null,
    from: filter.from ?? null,
    to: filter.to ?? null,
  };
}

/** Reads the two legs of a transfer, outgoing first as Rust returns them. */
export function legsFromWire(wire: readonly TransactionWire[]): TransferLegs {
  const [outgoing, incoming] = wire;
  if (wire.length !== 2 || outgoing === undefined || incoming === undefined) {
    throw new RepositoryError("internal", `Expected two transfer legs, received ${wire.length}.`);
  }
  return [transactionFromWire(outgoing), transactionFromWire(incoming)];
}
