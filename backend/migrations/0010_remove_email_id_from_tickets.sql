-- Remove email_id column from tickets table
ALTER TABLE tickets DROP COLUMN email_id;
DROP INDEX IF EXISTS tickets_email_id_idx; 