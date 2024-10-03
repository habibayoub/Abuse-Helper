CREATE TABLE customers (
  id SERIAL PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  ip VARCHAR(255)
);

INSERT INTO customers (id, email, ip) VALUES (0, 'john.smith@gmail.com', '192.0.0.1'); 