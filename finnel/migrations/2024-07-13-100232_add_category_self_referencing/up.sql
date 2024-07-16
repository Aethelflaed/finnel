-- Your SQL goes here
ALTER TABLE categories
ADD COLUMN parent_id BIGINT REFERENCES categories(id);
ALTER TABLE categories
ADD COLUMN replaced_by_id BIGINT REFERENCES categories(id);
