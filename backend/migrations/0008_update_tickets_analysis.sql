-- Add new columns for existing tickets
ALTER TABLE tickets 
    ADD COLUMN IF NOT EXISTS confidence_score FLOAT,
    ADD COLUMN IF NOT EXISTS identified_threats TEXT[],
    ADD COLUMN IF NOT EXISTS extracted_indicators TEXT[],
    ADD COLUMN IF NOT EXISTS analysis_summary TEXT;