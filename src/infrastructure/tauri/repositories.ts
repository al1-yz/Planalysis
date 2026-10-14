import type {
  AccountRepository,
  CardRepository,
  CategoryRepository,
  Repositories,
  TransactionRepository,
} from "../../domain/repositories";
import { call } from "./invoke";
import {
  accountFromWire,
  accountToWire,
  cardFromWire,
  cardToWire,
  categoryFromWire,
  categoryToWire,
  filterToWire,
  legsFromWire,
  transactionFromWire,
  transactionToWire,
  type AccountWire,
  type CardWire,
  type CategoryWire,
  type TransactionWire,
} from "./wire";

function accounts(): AccountRepository {
  return {
    async list() {
      return (await call<AccountWire[]>("account_list")).map(accountFromWire);
    },
    async get(id) {
      const wire = await call<AccountWire | null>("account_get", { id });
      return wire === null ? null : accountFromWire(wire);
    },
    async create(account) {
      await call("account_create", { account: accountToWire(account) });
    },
    async update(account) {
      await call("account_update", { account: accountToWire(account) });
    },
    async delete(id) {
      await call("account_delete", { id });
    },
  };
}

function categories(): CategoryRepository {
  return {
    async list() {
      return (await call<CategoryWire[]>("category_list")).map(categoryFromWire);
    },
    async get(id) {
      const wire = await call<CategoryWire | null>("category_get", { id });
      return wire === null ? null : categoryFromWire(wire);
    },
    async create(category) {
      await call("category_create", { category: categoryToWire(category) });
    },
    async update(category) {
      await call("category_update", { category: categoryToWire(category) });
    },
    async delete(id) {
      await call("category_delete", { id });
    },
  };
}

function cards(): CardRepository {
  return {
    async list() {
      return (await call<CardWire[]>("card_list")).map(cardFromWire);
    },
    async get(id) {
      const wire = await call<CardWire | null>("card_get", { id });
      return wire === null ? null : cardFromWire(wire);
    },
    async create(card) {
      await call("card_create", { card: cardToWire(card) });
    },
    async update(card) {
      await call("card_update", { card: cardToWire(card) });
    },
    async delete(id) {
      await call("card_delete", { id });
    },
  };
}

function transactions(): TransactionRepository {
  return {
    async list(filter) {
      const wire = await call<TransactionWire[]>("transaction_list", {
        filter: filterToWire(filter),
      });
      return wire.map(transactionFromWire);
    },
    async get(id) {
      const wire = await call<TransactionWire | null>("transaction_get", { id });
      return wire === null ? null : transactionFromWire(wire);
    },
    async create(transaction) {
      await call("transaction_create", { transaction: transactionToWire(transaction) });
    },
    async update(transaction) {
      await call("transaction_update", { transaction: transactionToWire(transaction) });
    },
    async delete(id) {
      await call("transaction_delete", { id });
    },
    async getTransfer(id) {
      const wire = await call<TransactionWire[] | null>("transfer_get", { transferId: id });
      return wire === null ? null : legsFromWire(wire);
    },
    async createTransfer(legs) {
      await call("transfer_create", { legs: legs.map(transactionToWire) });
    },
    async updateTransfer(legs) {
      await call("transfer_update", { legs: legs.map(transactionToWire) });
    },
    async deleteTransfer(id) {
      await call("transfer_delete", { transferId: id });
    },
  };
}

/** Repositories backed by the Rust commands of the desktop application. */
export function createTauriRepositories(): Repositories {
  return {
    accounts: accounts(),
    categories: categories(),
    cards: cards(),
    transactions: transactions(),
  };
}
