CREATE TABLE address (
  id INTEGER NOT NULL,
  coin TEXT NOT NULL,
  account TEXT NOT NULL,
  wallet_address TEXT NOT NULL,
  is_allocated INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT,
  CHECK ((account IN ('client', 'deposit', 'payment', 'stored'))),
  CHECK ((coin IN ('btc', 'bch', 'eth', 'xrp', 'hyt')))
);
