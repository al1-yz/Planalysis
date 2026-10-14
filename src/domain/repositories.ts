import type { Account } from "./account";
import type { LocalDate } from "./dates";
import type { Card } from "./card";
import type { Category } from "./category";
import type { AccountId, CardId, CategoryId, TransactionId, TransferId } from "./ids";
import type { Transaction } from "./transaction";
import type { TransferLegs } from "./transfer";

// Repositories take complete entities: the caller supplies ids and timestamps. Every write is
// atomic, and failures reject with a `RepositoryError`.

export interface AccountRepository {
  list(): Promise<Account[]>;
  get(id: AccountId): Promise<Account | null>;
  create(account: Account): Promise<void>;
  /** Replaces the account's fields. Setting `archivedAt` archives it. */
  update(account: Account): Promise<void>;
  /** Only succeeds when nothing refers to the account; otherwise archive it. */
  delete(id: AccountId): Promise<void>;
}

export interface CategoryRepository {
  list(): Promise<Category[]>;
  get(id: CategoryId): Promise<Category | null>;
  create(category: Category): Promise<void>;
  update(category: Category): Promise<void>;
  delete(id: CategoryId): Promise<void>;
}

export interface CardRepository {
  list(): Promise<Card[]>;
  get(id: CardId): Promise<Card | null>;
  create(card: Card): Promise<void>;
  update(card: Card): Promise<void>;
  delete(id: CardId): Promise<void>;
}

export interface TransactionFilter {
  accountId?: AccountId;
  cardId?: CardId;
  categoryId?: CategoryId;
  /** Inclusive lower bound on the transaction date. */
  from?: LocalDate;
  /** Inclusive upper bound on the transaction date. */
  to?: LocalDate;
}

export interface TransactionRepository {
  /** Newest first. */
  list(filter?: TransactionFilter): Promise<Transaction[]>;
  get(id: TransactionId): Promise<Transaction | null>;
  /** The single-transaction operations reject transfer legs; use the transfer operations. */
  create(transaction: Transaction): Promise<void>;
  update(transaction: Transaction): Promise<void>;
  delete(id: TransactionId): Promise<void>;

  getTransfer(id: TransferId): Promise<TransferLegs | null>;
  /** Writes both legs, or neither. */
  createTransfer(legs: TransferLegs): Promise<void>;
  /** Replaces both legs of the transfer they share, or neither. */
  updateTransfer(legs: TransferLegs): Promise<void>;
  /** Deletes both legs, or neither. */
  deleteTransfer(id: TransferId): Promise<void>;
}

/** Everything the application reads and writes, wired once at startup. */
export interface Repositories {
  accounts: AccountRepository;
  categories: CategoryRepository;
  cards: CardRepository;
  transactions: TransactionRepository;
}
