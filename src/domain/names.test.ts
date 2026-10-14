import { describe, expect, it } from "vitest";

import sharedCases from "../../contract/name-keys.json";
import { entityName, nameKey } from "./names";

const NBSP = String.fromCodePoint(0x00a0);
const IDEOGRAPHIC_SPACE = String.fromCodePoint(0x3000);
const COMBINING_ACUTE = String.fromCodePoint(0x0301);
const COMBINING_CEDILLA = String.fromCodePoint(0x0327);
const COMBINING_TILDE = String.fromCodePoint(0x0303);

describe("entityName", () => {
  it("trims and collapses runs of whitespace", () => {
    expect(entityName("  Conta   Corrente ")).toBe("Conta Corrente");
    expect(entityName("Nubank")).toBe("Nubank");
  });

  it("treats non-breaking and ideographic spaces as whitespace", () => {
    const raw = `${NBSP}Reserva${NBSP}${NBSP}de${IDEOGRAPHIC_SPACE}emergência${NBSP}`;
    expect(entityName(raw)).toBe("Reserva de emergência");
  });

  it("rejects a name that is empty after normalization", () => {
    expect(() => entityName("")).toThrow(RangeError);
    expect(() => entityName("   ")).toThrow(RangeError);
    expect(() => entityName(`${NBSP}\t\n`)).toThrow(RangeError);
  });
});

describe("nameKey", () => {
  it("ignores case, including accented letters", () => {
    expect(nameKey("Ação")).toBe("ação");
    expect(nameKey("AÇÃO")).toBe("ação");
    expect(nameKey("Saúde")).toBe(nameKey("SAÚDE"));
  });

  it("ignores spacing differences", () => {
    expect(nameKey("  Conta   Corrente ")).toBe("conta corrente");
    expect(nameKey(`Conta${NBSP}Corrente`)).toBe("conta corrente");
  });

  it("treats composed and decomposed accents as the same name", () => {
    const decomposed = `Ac${COMBINING_CEDILLA}a${COMBINING_TILDE}o`;
    expect(nameKey(decomposed)).toBe(nameKey("Ação"));
    expect(nameKey(`e${COMBINING_ACUTE}`)).toBe("é");
  });

  it("keeps accents significant", () => {
    expect(nameKey("Acao")).not.toBe(nameKey("Ação"));
  });

  it("is idempotent and rejects empty names", () => {
    const key = nameKey("  Cartão   NUBANK ");
    expect(key).toBe("cartão nubank");
    expect(nameKey(key)).toBe(key);
    expect(() => nameKey("  ")).toThrow(RangeError);
  });
});

describe("cases shared with the Rust side", () => {
  it.each(sharedCases.cases)("normalizes $key", ({ input, key }) => {
    expect(nameKey(input)).toBe(key);
  });
});
