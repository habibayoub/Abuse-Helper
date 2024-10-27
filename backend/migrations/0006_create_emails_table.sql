CREATE TABLE IF NOT EXISTS emails (
    id TEXT PRIMARY KEY,
    "from" TEXT NOT NULL,
    "to" TEXT[] NOT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX emails_from_idx ON emails("from");
CREATE INDEX emails_received_at_idx ON emails(received_at);
