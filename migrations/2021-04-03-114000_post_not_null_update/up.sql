UPDATE posts
SET updated = created
WHERE updated IS NULL;

ALTER TABLE posts ALTER COLUMN updated SET NOT NULL;
