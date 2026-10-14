// Whitespace follows the Unicode `White_Space` property so Rust's `char::is_whitespace` agrees.
const WHITESPACE_RUN = /\p{White_Space}+/gu;
const EDGE_WHITESPACE = /^\p{White_Space}+|\p{White_Space}+$/gu;

/** A user-facing name for an account, card or category: trimmed, single-spaced, never empty. */
export function entityName(raw: string): string {
  const name = raw.replace(EDGE_WHITESPACE, "").replace(WHITESPACE_RUN, " ");
  if (name.length === 0) {
    throw new RangeError("A name cannot be empty.");
  }
  return name;
}

/**
 * The comparison key behind "unique active names": the normalized name in Unicode NFC, lowercased
 * without regard to the device locale. Accents stay significant ("Acao" is not "Ação").
 */
export function nameKey(raw: string): string {
  return entityName(raw).normalize("NFC").toLowerCase();
}
