import { describe, expect, it } from "vitest";

import { CREATED_AT, IDS, UPDATED_AT } from "./entities.fixtures";
import { transactionId } from "./ids";
import { transaction } from "./transaction";
import { transfer, transferFromLegs, transferToLegs, type TransferInput } from "./transfer";

const input: TransferInput = {
  id: IDS.transfer,
  fromAccountId: IDS.account,
  toAccountId: IDS.otherAccount,
  amount: 25000,
  date: "2026-10-13",
  description: " Reserva ",
  createdAt: CREATED_AT,
  updatedAt: UPDATED_AT,
};

const legIds = [transactionId(IDS.transaction), transactionId(IDS.otherTransaction)] as const;

describe("transfer", () => {
  it("builds a transfer with a positive amount and a trimmed description", () => {
    expect(transfer(input)).toEqual({ ...input, description: "Reserva" });
  });

  it("rejects the same account on both sides and non-positive amounts", () => {
    expect(() => transfer({ ...input, toAccountId: IDS.account })).toThrow(RangeError);
    expect(() => transfer({ ...input, amount: 0 })).toThrow(RangeError);
    expect(() => transfer({ ...input, amount: -100 })).toThrow(RangeError);
    expect(() => transfer({ ...input, id: "nope" })).toThrow(RangeError);
  });
});

describe("transferToLegs", () => {
  it("creates an outgoing leg and an incoming leg with equal and opposite amounts", () => {
    const [outgoing, incoming] = transferToLegs(transfer(input), legIds);
    expect(outgoing).toMatchObject({
      id: IDS.transaction,
      accountId: IDS.account,
      amount: -25000,
      transferId: IDS.transfer,
      categoryId: null,
      cardId: null,
      date: "2026-10-13",
      description: "Reserva",
    });
    expect(incoming).toMatchObject({
      id: IDS.otherTransaction,
      accountId: IDS.otherAccount,
      amount: 25000,
      transferId: IDS.transfer,
      categoryId: null,
    });
  });
});

describe("transferFromLegs", () => {
  it("rebuilds the transfer from its legs in either order", () => {
    const legs = transferToLegs(transfer(input), legIds);
    expect(transferFromLegs(legs)).toEqual(transfer(input));
    expect(transferFromLegs([legs[1], legs[0]])).toEqual(transfer(input));
  });

  it("rejects legs that do not form a valid transfer", () => {
    const [outgoing, incoming] = transferToLegs(transfer(input), legIds);
    expect(() => transferFromLegs([outgoing])).toThrow(RangeError);
    expect(() => transferFromLegs([outgoing, incoming, incoming])).toThrow(RangeError);
    expect(() => transferFromLegs([outgoing, outgoing])).toThrow(RangeError);
    expect(() =>
      transferFromLegs([outgoing, { ...incoming, amount: 24999 as typeof incoming.amount }]),
    ).toThrow(RangeError);
    expect(() =>
      transferFromLegs([outgoing, { ...incoming, date: "2026-10-14" as typeof incoming.date }]),
    ).toThrow(RangeError);
    expect(() =>
      transferFromLegs([outgoing, { ...incoming, accountId: outgoing.accountId }]),
    ).toThrow(RangeError);
    expect(() => transferFromLegs([outgoing, { ...incoming, transferId: null }])).toThrow(
      RangeError,
    );
    expect(() => transferFromLegs([outgoing, { ...incoming, description: "Outra" }])).toThrow(
      RangeError,
    );
    const plain = transaction({
      id: IDS.otherTransaction,
      accountId: IDS.otherAccount,
      cardId: null,
      categoryId: null,
      transferId: null,
      date: "2026-10-13",
      amount: 25000,
      description: "Reserva",
      createdAt: CREATED_AT,
      updatedAt: UPDATED_AT,
    });
    expect(() => transferFromLegs([outgoing, plain])).toThrow(RangeError);
  });
});
