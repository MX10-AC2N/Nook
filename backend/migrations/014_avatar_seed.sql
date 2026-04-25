-- Add avatar_seed for user-selected specific avatar within a style
ALTER TABLE users ADD COLUMN avatar_seed TEXT DEFAULT NULL;
