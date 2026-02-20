CREATE TABLE all_types (
    id INTEGER PRIMARY KEY,
    small_val SMALLINT,
    big_val BIGINT,
    price NUMERIC(10, 2),
    ratio DOUBLE PRECISION,
    name VARCHAR(100),
    code CHAR(5),
    bio TEXT,
    active BOOLEAN,
    birthday DATE,
    login_time TIME,
    created_at TIMESTAMP,
    updated_at TIMESTAMPTZ,
    avatar BYTEA,
    external_id UUID,
    metadata JSONB,
    tags TEXT
);
