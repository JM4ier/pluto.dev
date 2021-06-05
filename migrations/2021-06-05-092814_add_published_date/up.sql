ALTER TABLE posts
ADD COLUMN published_at TIMESTAMP WITHOUT TIME ZONE;

UPDATE posts
SET published_at = created
WHERE published;

ALTER TABLE posts
DROP COLUMN published;

ALTER TABLE posts
RENAME COLUMN published_at TO published;
