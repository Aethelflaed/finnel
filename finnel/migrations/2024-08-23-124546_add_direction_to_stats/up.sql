-- Your SQL goes here
ALTER TABLE monthly_category_stats
ADD COLUMN direction TEXT NOT NULL DEFAULT 'Debit';

-- https://sqlite.org/lang_altertable.html#otheralter
-- 1. Foreign keys constraint are not enabled
-- 2. Already in a transaction
-- 3. No indexes, triggers or views
-- 4. Create the new table
CREATE TABLE new_monthly_stats (
  year INTEGER NOT NULL,
  month INTEGER NOT NULL,
  debit_amount BIGINT NOT NULL,
  credit_amount BIGINT NOT NULL,
  currency TEXT NOT NULL,
  CONSTRAINT monthly_stats_year_month PRIMARY KEY (year ASC, month ASC, currency ASC)
);
-- 5. Transfer content
INSERT INTO new_monthly_stats (
  year,
  month,
  debit_amount,
  credit_amount,
  currency
)
SELECT
  year,
  month,
  amount,
  0,
  currency
FROM monthly_stats;
-- 6. Drop the old table
DROP TABLE monthly_stats;
-- 7. Rename the table
ALTER TABLE new_monthly_stats RENAME TO monthly_stats;
-- 8. Re-create indexes, triggers, views
-- 9. Still no views
-- 10. Foreign key contraints probably wasn't enabled to start with
-- 11. Commit the transaction
-- 12. Foreign key contraints probably wasn't enabled to start with
