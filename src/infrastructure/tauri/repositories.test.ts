import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";

import accountFixture from "../../../contract/account.json";
import cardFixture from "../../../contract/card.json";
import categoryFixture from "../../../contract/category.json";
import filterFixture from "../../../contract/transaction-filter.json";
import transactionFixture from "../../../contract/transaction.json";
import transferLegsFixture from "../../../contract/transfer-legs.json";
import { account } from "../../domain/account";
import { card } from "../../domain/card";
import { category } from "../../domain/category";
import { localDate } from "../../domain/dates";
import { RepositoryError } from "../../domain/errors";
import { accountId, categoryId, transferId } from "../../domain/ids";
import { transaction } from "../../domain/transaction";
import { transferFromLegs, type TransferLegs } from "../../domain/transfer";
import { createTauriRepositories } from "./repositories";

type Call = { command: string; args: unknown };
let calls: Call[] = [];

/** Answers every command with `result` and records what was sent. */
function respondWith(result: unknown = null) {
  calls = [];
  mockIPC((command, args) => {
    calls.push({ command, args });
    return result;
  });
}

afterEach(() => {
  clearMocks();
});

const repositories = createTauriRepositories();
const legs = transferLegsFixture.map((leg) => transaction(leg)) as unknown as TransferLegs;

describe("accounts", () => {
  const entity = account(accountFixture);

  it("sends the entity with its name key, exactly as the shared fixture", async () => {
    respondWith();
    await repositories.accounts.create(entity);
    await repositories.accounts.update(entity);
    expect(calls).toEqual([
      { command: "account_create", args: { account: accountFixture } },
      { command: "account_update", args: { account: accountFixture } },
    ]);
  });

  it("reads and validates what Rust returns", async () => {
    respondWith([accountFixture]);
    await expect(repositories.accounts.list()).resolves.toEqual([entity]);
    respondWith(accountFixture);
    await expect(repositories.accounts.get(entity.id)).resolves.toEqual(entity);
    expect(calls).toEqual([{ command: "account_get", args: { id: entity.id } }]);
  });

  it("returns null for a missing account and passes the id to delete", async () => {
    respondWith(null);
    await expect(repositories.accounts.get(entity.id)).resolves.toBeNull();
    await repositories.accounts.delete(entity.id);
    expect(calls[1]).toEqual({ command: "account_delete", args: { id: entity.id } });
  });

  it("rejects data from Rust that breaks a domain rule", async () => {
    respondWith([{ ...accountFixture, type: "investment" }]);
    await expect(repositories.accounts.list()).rejects.toThrow(RangeError);
  });

  it("surfaces Rust failures as RepositoryError", async () => {
    mockIPC(() => {
      throw { code: "duplicate_name", message: "UNIQUE constraint failed" };
    });
    await expect(repositories.accounts.create(entity)).rejects.toEqual(
      expect.objectContaining({ name: "RepositoryError", code: "duplicate_name" }),
    );
  });
});

describe("categories", () => {
  const entity = category(categoryFixture);

  it("round-trips through the shared fixture", async () => {
    respondWith();
    await repositories.categories.create(entity);
    expect(calls[0]).toEqual({ command: "category_create", args: { category: categoryFixture } });
    respondWith([categoryFixture]);
    await expect(repositories.categories.list()).resolves.toEqual([entity]);
    respondWith(null);
    await expect(repositories.categories.get(entity.id)).resolves.toBeNull();
    await repositories.categories.update(entity);
    await repositories.categories.delete(entity.id);
    expect(calls.map((c) => c.command)).toEqual([
      "category_get",
      "category_update",
      "category_delete",
    ]);
  });
});

describe("cards", () => {
  const entity = card(cardFixture);

  it("round-trips through the shared fixture", async () => {
    respondWith();
    await repositories.cards.create(entity);
    expect(calls[0]).toEqual({ command: "card_create", args: { card: cardFixture } });
    respondWith([cardFixture]);
    await expect(repositories.cards.list()).resolves.toEqual([entity]);
    respondWith(null);
    await expect(repositories.cards.get(entity.id)).resolves.toBeNull();
    await repositories.cards.update(entity);
    await repositories.cards.delete(entity.id);
    expect(calls.map((c) => c.command)).toEqual(["card_get", "card_update", "card_delete"]);
  });
});

describe("transactions", () => {
  const entity = transaction(transactionFixture);

  it("sends and reads a transaction exactly as the shared fixture", async () => {
    respondWith();
    await repositories.transactions.create(entity);
    await repositories.transactions.update(entity);
    expect(calls).toEqual([
      { command: "transaction_create", args: { transaction: transactionFixture } },
      { command: "transaction_update", args: { transaction: transactionFixture } },
    ]);
    respondWith(transactionFixture);
    await expect(repositories.transactions.get(entity.id)).resolves.toEqual(entity);
    respondWith(null);
    await expect(repositories.transactions.get(entity.id)).resolves.toBeNull();
    await repositories.transactions.delete(entity.id);
    expect(calls[1]).toEqual({ command: "transaction_delete", args: { id: entity.id } });
  });

  it("sends a filter as the shared fixture, filling unset fields with null", async () => {
    respondWith([transactionFixture]);
    const listed = await repositories.transactions.list({
      accountId: accountId(filterFixture.accountId),
      from: localDate(filterFixture.from),
      to: localDate(filterFixture.to),
    });
    expect(listed).toEqual([entity]);
    expect(calls).toEqual([{ command: "transaction_list", args: { filter: filterFixture } }]);

    respondWith([]);
    await repositories.transactions.list();
    expect(calls[0]?.args).toEqual({
      filter: { accountId: null, cardId: null, categoryId: null, from: null, to: null },
    });
    await repositories.transactions.list({ categoryId: categoryId(categoryFixture.id) });
    expect(calls[1]?.args).toEqual({
      filter: {
        accountId: null,
        cardId: null,
        categoryId: categoryFixture.id,
        from: null,
        to: null,
      },
    });
  });
});

describe("transfers", () => {
  const id = transferId(transferLegsFixture[0]?.transferId ?? "");

  it("sends both legs together, exactly as the shared fixture", async () => {
    respondWith();
    await repositories.transactions.createTransfer(legs);
    await repositories.transactions.updateTransfer(legs);
    await repositories.transactions.deleteTransfer(id);
    expect(calls).toEqual([
      { command: "transfer_create", args: { legs: transferLegsFixture } },
      { command: "transfer_update", args: { legs: transferLegsFixture } },
      { command: "transfer_delete", args: { transferId: id } },
    ]);
  });

  it("reads a transfer back as two valid legs", async () => {
    respondWith(transferLegsFixture);
    const read = await repositories.transactions.getTransfer(id);
    expect(read).toEqual(legs);
    expect(read && transferFromLegs(read).amount).toBe(25000);
    expect(calls).toEqual([{ command: "transfer_get", args: { transferId: id } }]);
  });

  it("returns null for an unknown transfer and rejects a malformed one", async () => {
    respondWith(null);
    await expect(repositories.transactions.getTransfer(id)).resolves.toBeNull();
    respondWith([transferLegsFixture[0]]);
    await expect(repositories.transactions.getTransfer(id)).rejects.toBeInstanceOf(RepositoryError);
  });
});
