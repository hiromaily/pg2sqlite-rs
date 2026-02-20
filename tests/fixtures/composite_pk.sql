CREATE TABLE user_roles (
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    granted_at TIMESTAMP DEFAULT now(),
    PRIMARY KEY (user_id, role_id)
);
