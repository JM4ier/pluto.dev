INSERT INTO tags_meta (tag, display, description)
SELECT DISTINCT tag as rel_tag, true, ''
FROM tags
ON CONFLICT DO NOTHING;

ALTER TABLE tags
ADD CONSTRAINT fk_tag
FOREIGN KEY (tag)
REFERENCES tags_meta (tag);
