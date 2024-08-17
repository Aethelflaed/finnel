-- Your SQL goes here
CREATE TABLE monthly_stats (
  year INTEGER NOT NULL,
  month INTEGER NOT NULL,
  amount BIGINT NOT NULL,
  currency TEXT NOT NULL,
  CONSTRAINT monthly_stats_year_month PRIMARY KEY (year ASC, month ASC, currency ASC)
);

CREATE TABLE monthly_category_stats (
  id INTEGER NOT NULL PRIMARY KEY,
  year INTEGER NOT NULL,
  month INTEGER NOT NULL,
  amount BIGINT NOT NULL,
  currency TEXT NOT NULL,
  category_id BIGINT REFERENCES categories(id),
  FOREIGN KEY (year, month, currency) REFERENCES monthly_stats(year, month, currency)
);
