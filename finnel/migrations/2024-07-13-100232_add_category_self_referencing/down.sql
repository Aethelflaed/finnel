-- This file should undo anything in `up.sql`
ALTER TABLE categories
DROP COLUMN replaced_by_id;
ALTER TABLE categories
DROP COLUMN parent_id;
