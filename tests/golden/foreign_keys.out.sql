CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL,
  total NUMERIC,
  created_at TEXT DEFAULT (CURRENT_TIMESTAMP)
);

CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE INDEX idx_orders_user ON orders (user_id);
