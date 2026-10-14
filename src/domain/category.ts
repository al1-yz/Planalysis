import { categoryId, type CategoryId } from "./ids";
import { entityName } from "./names";
import { timestamp, type Timestamp } from "./timestamp";

export const CATEGORY_KINDS = ["income", "expense"] as const;
export type CategoryKind = (typeof CATEGORY_KINDS)[number];

/** Says what a transaction was for. An explicit category decides income vs expense. */
export interface Category {
  id: CategoryId;
  name: string;
  kind: CategoryKind;
  archivedAt: Timestamp | null;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

/** Unvalidated category fields, as received from storage or a form. */
export interface CategoryInput {
  id: string;
  name: string;
  kind: string;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

/** Validates raw fields into a {@link Category}. Throws `RangeError` on any invalid field. */
export function category(input: CategoryInput): Category {
  const kind = CATEGORY_KINDS.find((candidate) => candidate === input.kind);
  if (kind === undefined) {
    throw new RangeError(`Invalid category kind "${input.kind}".`);
  }
  return {
    id: categoryId(input.id),
    name: entityName(input.name),
    kind,
    archivedAt: input.archivedAt === null ? null : timestamp(input.archivedAt),
    createdAt: timestamp(input.createdAt),
    updatedAt: timestamp(input.updatedAt),
  };
}
