-- Add argon2_salt column to store password derivation salt
ALTER TABLE aead_encrypted ADD COLUMN argon2_salt TEXT;
