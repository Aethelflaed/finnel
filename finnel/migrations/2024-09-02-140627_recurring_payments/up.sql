-- Your SQL goes here
CREATE TABLE recurring_payments (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  frequency TEXT NOT NULL,
  account_id BIGINT NOT NULL REFERENCES accounts(id),
  amount BIGINT NOT NULL,
  currency TEXT NOT NULL,
  direction TEXT NOT NULL,
  mode TEXT NOT NULL,
  category_id BIGINT REFERENCES categories(id),
  merchant_id BIGINT REFERENCES merchants(id)
);
