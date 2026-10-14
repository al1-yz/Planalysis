import { moneyCents, type MoneyCents } from "./money";

/**
 * Splits `total` into `count` installments that always sum back to `total`. Whole-cent shares
 * are equal; the remainder cents go one each to the first installments. A negative total (a
 * refund) is split the same way in magnitude, so its extra cents are also on the first ones.
 * Throws `RangeError` when `count` is not a positive integer.
 */
export function allocateInstallments(total: MoneyCents, count: number): MoneyCents[] {
  if (!Number.isSafeInteger(count) || count < 1) {
    throw new RangeError(`Installment count must be a positive integer, received ${count}.`);
  }
  // `%` keeps the sign of the dividend, so `remainder` and `share` agree with the total's sign.
  const remainder = total % count;
  const share = (total - remainder) / count;
  const extraCent = Math.sign(remainder);
  return Array.from({ length: count }, (_, index) =>
    moneyCents(index < Math.abs(remainder) ? share + extraCent : share),
  );
}
