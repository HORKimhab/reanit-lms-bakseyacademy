-- Reviews are editable by their author; track when they last changed.
ALTER TABLE reviews ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
UPDATE reviews SET updated_at = created_at;
