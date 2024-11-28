CREATE TABLE IF NOT EXISTS emails (
    id UUID PRIMARY KEY,
    sender TEXT NOT NULL,
    recipients TEXT[] NOT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    analyzed BOOLEAN DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS emails_sender_idx ON emails(sender);
CREATE INDEX IF NOT EXISTS emails_received_at_idx ON emails(received_at);
CREATE INDEX IF NOT EXISTS emails_analyzed_idx ON emails(analyzed);
