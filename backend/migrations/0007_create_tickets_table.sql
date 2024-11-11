CREATE TABLE IF NOT EXISTS tickets (
    id UUID PRIMARY KEY,
    ticket_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Open',
    ip_address TEXT,
    email_id TEXT NOT NULL REFERENCES emails(id),
    subject TEXT NOT NULL,
    description TEXT NOT NULL,
    confidence_score FLOAT,
    identified_threats TEXT[],
    extracted_indicators TEXT[],
    analysis_summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX tickets_ip_address_idx ON tickets(ip_address);
CREATE INDEX tickets_email_id_idx ON tickets(email_id);
CREATE INDEX tickets_ticket_type_idx ON tickets(ticket_type);
CREATE INDEX tickets_status_idx ON tickets(status);
