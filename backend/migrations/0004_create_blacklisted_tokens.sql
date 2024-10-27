CREATE TABLE blacklisted_tokens (
    uuid UUID PRIMARY KEY,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blacklisted_tokens_token ON blacklisted_tokens (token);
CREATE INDEX idx_blacklisted_tokens_expires_at ON blacklisted_tokens (expires_at);