CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL
);

-- Add a default admin user (password: admin123)
-- Note: In a real-world scenario, you'd want to use a more secure password and possibly set it up differently
INSERT INTO users (email, password_hash, role)
VALUES ('admin@example.com', '$2b$12$1234567890123456789012uQGZXvmF7Ue8Yd9O9Yt1zIH1QG1Aw5S', 'admin');