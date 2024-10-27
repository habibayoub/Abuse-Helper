CREATE TYPE ticket_type AS ENUM ('Malware', 'Phishing', 'Scam', 'Spam', 'Other');
CREATE TYPE ticket_status AS ENUM ('Open', 'InProgress', 'Closed', 'Resolved');

CREATE TABLE IF NOT EXISTS tickets (
    id UUID PRIMARY KEY,
    ticket_type ticket_type NOT NULL,
    status ticket_status NOT NULL DEFAULT 'Open',
    ip_address TEXT,
    email_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX tickets_ip_address_idx ON tickets(ip_address);
CREATE INDEX tickets_email_id_idx ON tickets(email_id);
