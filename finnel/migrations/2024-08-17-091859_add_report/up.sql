-- Your SQL goes here
CREATE TABLE reports (
  id INTEGER NOT NULL PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);

CREATE TABLE reports_categories (
  report_id BIGINT REFERENCES reports(id) NOT NULL,
  category_id BIGINT REFERENCES categories(id) NOT NULL,
  CONSTRAINT reports_categories_pk PRIMARY KEY (report_id, category_id)
);
