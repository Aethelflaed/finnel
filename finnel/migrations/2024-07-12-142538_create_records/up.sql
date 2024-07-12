-- Your SQL goes here
CREATE TABLE IF NOT EXISTS records (
  id INTEGER NOT NULL PRIMARY KEY,
  account_id BIGINT NOT NULL REFERENCES accounts(id),
  amount BIGINT NOT NULL,
  currency TEXT NOT NULL,
  operation_date DATETIME NOT NULL,
  value_date DATETIME NOT NULL,
  direction TEXT NOT NULL DEFAULT 'Debit',
  mode TEXT NOT NULL DEFAULT 'Direct',
  details TEXT NOT NULL DEFAULT '',
  category_id BIGINT REFERENCES categories(id),
  merchant_id BIGINT REFERENCES merchants(id)
);
