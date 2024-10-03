CREATE TABLE blacklisted_tokens (
    id SERIAL PRIMARY KEY,
    token TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blacklisted_tokens_token ON blacklisted_tokens (token);