CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL
);

INSERT INTO users (email, password_hash, role)
VALUES ('admin@example.com', '$2a$10$qfOU30x3oKkpQZiuIjQWyOlvAL2gyX3pl0taFUBmYfRM0qmAM6bFC', 'admin');