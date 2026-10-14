/** Integer BRL cents. `R$ 10,50` is `1050`. Create values with {@link moneyCents}. */
export type MoneyCents = number & { readonly __brand: "MoneyCents" };

/** Validates that `value` is a safe integer and brands it as {@link MoneyCents}. */
export function moneyCents(value: number): MoneyCents {
  if (!Number.isSafeInteger(value)) {
    throw new RangeError(`Money must be a safe integer number of cents, received ${value}.`);
  }
  // `+ 0` turns -0 into 0 so equality checks never see a negative zero.
  return (value + 0) as MoneyCents;
}

/** Adds two amounts; throws `RangeError` if the result leaves the safe integer range. */
export function addCents(a: MoneyCents, b: MoneyCents): MoneyCents {
  return moneyCents(a + b);
}

/** Subtracts `b` from `a`; throws `RangeError` if the result leaves the safe integer range. */
export function subtractCents(a: MoneyCents, b: MoneyCents): MoneyCents {
  return moneyCents(a - b);
}

/** Sums amounts, validating every running total so an overflow is never masked by a later term. */
export function sumCents(values: readonly MoneyCents[]): MoneyCents {
  return values.reduce(addCents, moneyCents(0));
}
