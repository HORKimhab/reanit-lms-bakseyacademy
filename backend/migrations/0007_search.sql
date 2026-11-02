-- Fuzzy catalog search: trigram indexes speed up ILIKE '%q%' and power typo-tolerant `<%` matches.
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX courses_title_trgm_idx     ON courses     USING gin (title gin_trgm_ops);
CREATE INDEX instructors_name_trgm_idx  ON instructors USING gin (name gin_trgm_ops);
CREATE INDEX ebooks_title_trgm_idx      ON ebooks      USING gin (title gin_trgm_ops);
CREATE INDEX ebooks_author_trgm_idx     ON ebooks      USING gin (author_name gin_trgm_ops);
