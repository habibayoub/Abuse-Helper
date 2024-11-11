CREATE TABLE IF NOT EXISTS emails (
    id TEXT PRIMARY KEY,
    sender TEXT NOT NULL,
    recipients TEXT[] NOT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    analyzed BOOLEAN DEFAULT FALSE
);

CREATE INDEX emails_sender_idx ON emails(sender);
CREATE INDEX emails_received_at_idx ON emails(received_at);
CREATE INDEX emails_analyzed_idx ON emails(analyzed);
