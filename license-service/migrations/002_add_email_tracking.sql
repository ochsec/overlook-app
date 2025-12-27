-- Add email tracking columns
ALTER TABLE licenses ADD COLUMN email_sent BOOLEAN DEFAULT FALSE;
ALTER TABLE licenses ADD COLUMN email_sent_at TEXT;
ALTER TABLE licenses ADD COLUMN email_send_attempts INTEGER DEFAULT 0;
ALTER TABLE licenses ADD COLUMN email_last_error TEXT;

-- Create index for finding licenses with failed emails
CREATE INDEX IF NOT EXISTS idx_email_sent ON licenses(email_sent);
