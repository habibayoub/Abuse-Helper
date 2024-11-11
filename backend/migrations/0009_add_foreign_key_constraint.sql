-- Add foreign key constraint to tickets table
ALTER TABLE tickets
    ADD CONSTRAINT fk_tickets_email
    FOREIGN KEY (email_id)
    REFERENCES emails(id)
    ON DELETE CASCADE; 