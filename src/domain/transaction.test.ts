import { describe, expect, it } from "vitest";

import { CREATED_AT, IDS, UPDATED_AT } from "./entities.fixtures";
import { transaction, type TransactionInput } from "./transaction";

const onAccount: TransactionInput = {
  id: IDS.transaction,
  accountId: IDS.account,
  cardId: null,
  categoryId: IDS.category,
  transferId: null,
  date: "2026-10-13",
  amount: -4590,
  description: "  Mercado  ",
  createdAt: CREATED_AT,
  updatedAt: UPDATED_AT,
};

describe("transaction", () => {
  it("builds a signed transaction on an account with a trimmed description", () => {
    expect(transaction(onAccount)).toEqual({ ...onAccount, description: "Mercado" });
    expect(transaction({ ...onAccount, amount: 100000 }).amount).toBe(100000);
  });

  it("allows an uncategorized transaction and an empty description", () => {
    const built = transaction({ ...onAccount, categoryId: null, description: "" });
    expect(built.categoryId).toBeNull();
    expect(built.description).toBe("");
  });

  it("builds a purchase on a card", () => {
    const built = transaction({ ...onAccount, accountId: null, cardId: IDS.card });
    expect(built.cardId).toBe(IDS.card);
    expect(built.accountId).toBeNull();
  });

  it("builds a transfer leg: an account, a transfer id and no category", () => {
    const leg = transaction({ ...onAccount, categoryId: null, transferId: IDS.transfer });
    expect(leg.transferId).toBe(IDS.transfer);
  });

  it("requires exactly one of account or card", () => {
    expect(() => transaction({ ...onAccount, accountId: null })).toThrow(RangeError);
    expect(() => transaction({ ...onAccount, cardId: IDS.card })).toThrow(RangeError);
  });

  it("rejects a transfer leg that has a card or a category", () => {
    const leg = { ...onAccount, categoryId: null, transferId: IDS.transfer };
    expect(() => transaction({ ...leg, categoryId: IDS.category })).toThrow(RangeError);
    expect(() => transaction({ ...leg, accountId: null, cardId: IDS.card })).toThrow(RangeError);
  });

  it("rejects a zero or fractional amount and invalid fields", () => {
    expect(() => transaction({ ...onAccount, amount: 0 })).toThrow(RangeError);
    expect(() => transaction({ ...onAccount, amount: 10.5 })).toThrow(RangeError);
    expect(() => transaction({ ...onAccount, id: "nope" })).toThrow(RangeError);
    expect(() => transaction({ ...onAccount, date: "2026-02-30" })).toThrow(RangeError);
    expect(() => transaction({ ...onAccount, categoryId: "nope" })).toThrow(RangeError);
    expect(() => transaction({ ...onAccount, createdAt: "x" })).toThrow(RangeError);
  });
});
