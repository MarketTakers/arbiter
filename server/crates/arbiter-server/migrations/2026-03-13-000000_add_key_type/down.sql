-- Not reversible without data loss; drop the column to revert
ALTER TABLE useragent_client DROP COLUMN key_type;
