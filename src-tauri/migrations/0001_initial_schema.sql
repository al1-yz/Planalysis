-- Initial Planalysis schema.
--
-- Money is integer cents (|value| <= 2^53 - 1, the JavaScript safe-integer limit). Calendar dates
-- are 'YYYY-MM-DD' text and timestamps are ISO-8601 UTC text. `date(x) IS x` rejects malformed
-- or impossible dates; `IS` is deliberate because a CHECK that evaluates to NULL would pass.
-- `name_key` is the normalized comparison key computed by the application; uniqueness of active
-- names is enforced on it so archived names can be reused.

CREATE TABLE accounts (
  id                    TEXT    PRIMARY KEY CHECK (id GLOB '????????-????-????-????-????????????'),
  name                  TEXT    NOT NULL CHECK (length(trim(name)) > 0),
  name_key              TEXT    NOT NULL CHECK (length(name_key) > 0),
  type                  TEXT    NOT NULL CHECK (type IN ('checking', 'savings', 'cash')),
  opening_balance_cents INTEGER NOT NULL CHECK (abs(opening_balance_cents) <= 9007199254740991),
  opening_date          TEXT    NOT NULL CHECK (date(opening_date) IS opening_date AND opening_date >= '0001-01-01'),
  archived_at           TEXT    CHECK (archived_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  created_at            TEXT    NOT NULL CHECK (created_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  updated_at            TEXT    NOT NULL CHECK (updated_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z')
) STRICT;

CREATE UNIQUE INDEX ux_accounts_active_name ON accounts (name_key) WHERE archived_at IS NULL;

CREATE TABLE categories (
  id          TEXT PRIMARY KEY CHECK (id GLOB '????????-????-????-????-????????????'),
  name        TEXT NOT NULL CHECK (length(trim(name)) > 0),
  name_key    TEXT NOT NULL CHECK (length(name_key) > 0),
  kind        TEXT NOT NULL CHECK (kind IN ('income', 'expense')),
  archived_at TEXT CHECK (archived_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  created_at  TEXT NOT NULL CHECK (created_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  updated_at  TEXT NOT NULL CHECK (updated_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z')
) STRICT;

CREATE UNIQUE INDEX ux_categories_active_name ON categories (kind, name_key) WHERE archived_at IS NULL;

CREATE TABLE cards (
  id                 TEXT    PRIMARY KEY CHECK (id GLOB '????????-????-????-????-????????????'),
  name               TEXT    NOT NULL CHECK (length(trim(name)) > 0),
  name_key           TEXT    NOT NULL CHECK (length(name_key) > 0),
  closing_day        INTEGER NOT NULL CHECK (closing_day BETWEEN 1 AND 31),
  due_day            INTEGER NOT NULL CHECK (due_day BETWEEN 1 AND 31),
  credit_limit_cents INTEGER CHECK (credit_limit_cents BETWEEN 0 AND 9007199254740991),
  archived_at        TEXT    CHECK (archived_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  created_at         TEXT    NOT NULL CHECK (created_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  updated_at         TEXT    NOT NULL CHECK (updated_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z')
) STRICT;

CREATE UNIQUE INDEX ux_cards_active_name ON cards (name_key) WHERE archived_at IS NULL;

-- A transaction belongs to exactly one account or one card. Both legs of a transfer share a
-- transfer_id, sit on accounts and carry no category. Pair rules (two legs, equal and opposite,
-- same date) are verified by the application inside the writing transaction.
CREATE TABLE transactions (
  id           TEXT    PRIMARY KEY CHECK (id GLOB '????????-????-????-????-????????????'),
  account_id   TEXT    REFERENCES accounts   (id) ON DELETE RESTRICT,
  card_id      TEXT    REFERENCES cards      (id) ON DELETE RESTRICT,
  category_id  TEXT    REFERENCES categories (id) ON DELETE RESTRICT,
  transfer_id  TEXT    CHECK (transfer_id GLOB '????????-????-????-????-????????????'),
  occurred_on  TEXT    NOT NULL CHECK (date(occurred_on) IS occurred_on AND occurred_on >= '0001-01-01'),
  amount_cents INTEGER NOT NULL CHECK (amount_cents <> 0 AND abs(amount_cents) <= 9007199254740991),
  description  TEXT    NOT NULL,
  created_at   TEXT    NOT NULL CHECK (created_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  updated_at   TEXT    NOT NULL CHECK (updated_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]T*Z'),
  CHECK ((account_id IS NULL) <> (card_id IS NULL)),
  CHECK (transfer_id IS NULL OR (account_id IS NOT NULL AND category_id IS NULL))
) STRICT;

CREATE INDEX ix_transactions_account_date  ON transactions (account_id,  occurred_on);
CREATE INDEX ix_transactions_card_date     ON transactions (card_id,     occurred_on);
CREATE INDEX ix_transactions_category_date ON transactions (category_id, occurred_on);
CREATE INDEX ix_transactions_date          ON transactions (occurred_on);

-- At most one leg per account within a transfer (rows without a transfer_id are exempt).
CREATE UNIQUE INDEX ux_transactions_transfer_leg ON transactions (transfer_id, account_id);

-- A transaction can never be dated before its account's opening date.
CREATE TRIGGER tr_transactions_opening_date_insert
BEFORE INSERT ON transactions
WHEN NEW.account_id IS NOT NULL
 AND NEW.occurred_on < (SELECT opening_date FROM accounts WHERE id = NEW.account_id)
BEGIN
  SELECT RAISE(ABORT, 'before_opening_date');
END;

CREATE TRIGGER tr_transactions_opening_date_update
BEFORE UPDATE OF occurred_on, account_id ON transactions
WHEN NEW.account_id IS NOT NULL
 AND NEW.occurred_on < (SELECT opening_date FROM accounts WHERE id = NEW.account_id)
BEGIN
  SELECT RAISE(ABORT, 'before_opening_date');
END;

-- The opening date can never move past an existing transaction.
CREATE TRIGGER tr_accounts_opening_date_update
BEFORE UPDATE OF opening_date ON accounts
WHEN EXISTS (SELECT 1 FROM transactions WHERE account_id = NEW.id AND occurred_on < NEW.opening_date)
BEGIN
  SELECT RAISE(ABORT, 'opening_date_after_transactions');
END;
