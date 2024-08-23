-- This file should undo anything in `up.sql`
ALTER TABLE monthly_stats
RENAME COLUMN debit_amount TO amount;
ALTER TABLE monthly_stats
DROP COLUMN credit_amount;
ALTER TABLE monthly_category_stats
DROP COLUMN direction;
