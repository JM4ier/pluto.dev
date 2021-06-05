-- ALTER TABLE posts
-- ADD COLUMN published_at TIMESTAMP WITHOUT TIME ZONE;
-- 
-- UPDATE posts
-- SET published_at = created
-- WHERE published;
-- 
-- ALTER TABLE posts
-- DROP COLUMN published;
-- 
-- ALTER TABLE posts
-- RENAME COLUMN published_at TO published;

ALTER TABLE posts
ADD COLUMN is_published BOOLEAN NOT NULL DEFAULT 'f';

UPDATE posts
SET is_published = 't'
WHERE published IS NOT NULL;

ALTER TABLE posts
DROP COLUMN published;

ALTER TABLE posts
RENAME COLUMN is_published TO published;
