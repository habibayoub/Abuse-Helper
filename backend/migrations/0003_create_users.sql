CREATE TABLE IF NOT EXISTS users (
    uuid UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users (uuid, email, name, password_hash, role)
VALUES (
    '00000000-0000-0000-0000-000000000000',
    'admin@example.com',
    'Admin User',
    '$2a$10$qfOU30x3oKkpQZiuIjQWyOlvAL2gyX3pl0taFUBmYfRM0qmAM6bFC',
    'admin'
);
