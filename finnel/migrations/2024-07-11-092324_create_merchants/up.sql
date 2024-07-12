-- Your SQL goes here
CREATE TABLE merchants (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  default_category_id BIGINT REFERENCES categories(id)
);
