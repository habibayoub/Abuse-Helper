-- Create junction table for email-ticket relationship
CREATE TABLE IF NOT EXISTS email_tickets (
    email_id UUID REFERENCES emails(id) ON DELETE RESTRICT,
    ticket_id UUID REFERENCES tickets(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (email_id, ticket_id)
);

-- Add indices for better query performance
CREATE INDEX IF NOT EXISTS email_tickets_email_id_idx ON email_tickets(email_id);
CREATE INDEX IF NOT EXISTS email_tickets_ticket_id_idx ON email_tickets(ticket_id);

-- Migrate existing relationships
DO $$
DECLARE
    t RECORD;
BEGIN
    -- Migrate existing relationships
    FOR t IN (
        SELECT id, email_id::uuid as email_id
        FROM tickets
        WHERE email_id IS NOT NULL
          AND email_id != ''
          AND EXISTS (SELECT 1 FROM emails e WHERE e.id = tickets.email_id::uuid)
    ) LOOP
        INSERT INTO email_tickets (email_id, ticket_id)
        VALUES (t.email_id, t.id)
        ON CONFLICT DO NOTHING;
    END LOOP;
END $$; 