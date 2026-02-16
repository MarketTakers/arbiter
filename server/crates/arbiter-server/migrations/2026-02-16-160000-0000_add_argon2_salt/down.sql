-- Remove argon2_salt column
ALTER TABLE aead_encrypted DROP COLUMN argon2_salt;
