ALTER TABLE minecraft_users
ADD COLUMN account_type TEXT NOT NULL DEFAULT 'microsoft'
CHECK (account_type IN ('microsoft', 'offline'));
