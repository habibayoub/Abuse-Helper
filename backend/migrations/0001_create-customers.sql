CREATE TABLE IF NOT EXISTS customers (
  uuid UUID PRIMARY KEY,
  email VARCHAR(255) NOT NULL UNIQUE,
  first_name VARCHAR(255),
  last_name VARCHAR(255),
  ip VARCHAR(45),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO customers (uuid, email, first_name, last_name, ip) VALUES ('123e4567-e89b-12d3-a456-426614175306', 'john.smith@gmail.com', 'John', 'Smith', '192.0.0.1');
