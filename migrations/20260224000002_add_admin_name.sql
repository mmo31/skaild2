-- Add name column to admins table
ALTER TABLE admins ADD COLUMN name VARCHAR(255) NOT NULL DEFAULT 'Admin';

-- Update the default for future rows (optional, keeps existing behavior)
ALTER TABLE admins ALTER COLUMN name DROP DEFAULT;
