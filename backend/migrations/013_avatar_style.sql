-- Add avatar_style column for DiceBear style preference
ALTER TABLE users ADD COLUMN avatar_style TEXT DEFAULT 'adventurer';
