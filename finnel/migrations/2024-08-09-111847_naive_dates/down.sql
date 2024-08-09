-- https://sqlite.org/lang_altertable.html#otheralter
-- 1. Foreign keys constraint are not enabled
-- 2. Already in a transaction
-- 3. No indexes, triggers or views
-- 4. Create the new table
CREATE TABLE new_records (
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
-- 5. Transfer content
INSERT INTO new_records (
  account_id,
  amount,
  currency,
  operation_date,
  value_date,
  direction,
  mode,
  details,
  category_id,
  merchant_id
)
SELECT
  account_id,
  amount,
  currency, 
  datetime(operation_date),
  datetime(value_date),
  direction,
  mode,
  details,
  category_id,
  merchant_id
FROM records;
-- 6. Drop the old table
DROP TABLE records;
-- 7. Rename the table
ALTER TABLE new_records RENAME TO records;
-- 8. Re-create indexes, triggers, views
-- 9. Still no views
-- 10. Foreign key contraints probably wasn't enabled to start with
-- 11. Commit the transaction
-- 12. Foreign key contraints probably wasn't enabled to start with
