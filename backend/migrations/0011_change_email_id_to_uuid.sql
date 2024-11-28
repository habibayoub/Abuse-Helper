-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- First, create a temporary column for the new UUID
ALTER TABLE emails ADD COLUMN new_id UUID;

-- Generate UUIDs for existing rows
UPDATE emails SET new_id = uuid_generate_v4();

-- Update email_tickets table to use UUID for email_id
ALTER TABLE email_tickets ADD COLUMN new_email_id UUID;
UPDATE email_tickets SET new_email_id = emails.new_id FROM emails WHERE email_tickets.email_id = emails.id;

-- Drop old foreign key constraint
ALTER TABLE email_tickets DROP CONSTRAINT IF EXISTS email_tickets_email_id_fkey;

-- Drop old primary key constraint
ALTER TABLE emails DROP CONSTRAINT IF EXISTS emails_pkey;

-- Drop old columns
ALTER TABLE email_tickets DROP COLUMN email_id;
ALTER TABLE emails DROP COLUMN id;

-- Rename new columns
ALTER TABLE emails RENAME COLUMN new_id TO id;
ALTER TABLE email_tickets RENAME COLUMN new_email_id TO email_id;

-- Add primary key and foreign key constraints
ALTER TABLE emails ADD PRIMARY KEY (id);
ALTER TABLE email_tickets ADD CONSTRAINT email_tickets_email_id_fkey 
    FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE RESTRICT;

-- Recreate indices if needed
DROP INDEX IF EXISTS email_tickets_email_id_idx;
CREATE INDEX IF NOT EXISTS email_tickets_email_id_idx ON email_tickets(email_id); 