/** Identifier of an {@link import("./account").Account}. A canonical lowercase UUID. */
export type AccountId = string & { readonly __brand: "AccountId" };
/** Identifier of a {@link import("./card").Card}. A canonical lowercase UUID. */
export type CardId = string & { readonly __brand: "CardId" };
/** Identifier of a {@link import("./category").Category}. A canonical lowercase UUID. */
export type CategoryId = string & { readonly __brand: "CategoryId" };
/** Identifier of a {@link import("./transaction").Transaction}. A canonical lowercase UUID. */
export type TransactionId = string & { readonly __brand: "TransactionId" };
/** Links the two legs of a transfer. A canonical lowercase UUID. */
export type TransferId = string & { readonly __brand: "TransferId" };

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;

function uuid(value: string, label: string): string {
  if (!UUID_PATTERN.test(value)) {
    throw new RangeError(`Invalid ${label} "${value}": expected a canonical lowercase UUID.`);
  }
  return value;
}

export function accountId(value: string): AccountId {
  return uuid(value, "AccountId") as AccountId;
}

export function cardId(value: string): CardId {
  return uuid(value, "CardId") as CardId;
}

export function categoryId(value: string): CategoryId {
  return uuid(value, "CategoryId") as CategoryId;
}

export function transactionId(value: string): TransactionId {
  return uuid(value, "TransactionId") as TransactionId;
}

export function transferId(value: string): TransferId {
  return uuid(value, "TransferId") as TransferId;
}
