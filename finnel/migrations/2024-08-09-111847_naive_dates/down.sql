-- Rename the date columns
ALTER TABLE records
RENAME COLUMN operation_date TO naive_operation_date;
ALTER TABLE records
RENAME COLUMN value_date TO naive_value_date;
-- Add the datetime columns
ALTER TABLE records
ADD COLUMN operation_date DATETIME NOT NULL;
ALTER TABLE records
ADD COLUMN value_date DATETIME NOT NULL;
-- Populate the new datetime columns
UPDATE records
SET
  operation_date = datetime(naive_operation_date),
  value_date = datetime(naive_value_date)
;
-- Drop the date columns
ALTER TABLE records
DROP COLUMN naive_operation_date;
ALTER TABLE records
DROP COLUMN naive_value_date;
