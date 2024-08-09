-- Rename the datetime columns
ALTER TABLE records
RENAME COLUMN operation_date TO operation_datetime;
ALTER TABLE records
RENAME COLUMN value_date TO value_datetime;
-- Add the date columns
ALTER TABLE records
ADD COLUMN operation_date DATE NOT NULL;
ALTER TABLE records
ADD COLUMN value_date DATE NOT NULL;
-- Populate the new date columns
UPDATE records
SET
  operation_date = date(operation_datetime),
  value_date = date(value_datetime)
;
-- Drop the datetime columns
ALTER TABLE records
DROP COLUMN operation_datetime;
ALTER TABLE records
DROP COLUMN value_datetime;
