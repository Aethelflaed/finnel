-- This file should undo anything in `up.sql`
ALTER TABLE merchants
DROP COLUMN replaced_by_id;
