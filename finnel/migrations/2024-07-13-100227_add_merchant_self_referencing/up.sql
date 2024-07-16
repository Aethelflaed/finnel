-- Your SQL goes here
ALTER TABLE merchants
ADD COLUMN replaced_by_id BIGINT REFERENCES merchants(id);
