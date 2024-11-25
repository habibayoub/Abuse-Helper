-- Add is_sent column
ALTER TABLE emails ADD COLUMN IF NOT EXISTS is_sent BOOLEAN NOT NULL DEFAULT FALSE;

-- Add index for is_sent
CREATE INDEX IF NOT EXISTS emails_is_sent_idx ON emails(is_sent); 