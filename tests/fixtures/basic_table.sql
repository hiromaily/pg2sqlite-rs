CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email VARCHAR(255) UNIQUE,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT now()
);
