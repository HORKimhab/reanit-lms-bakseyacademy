-- Uploaded media (lesson videos, ebook files). Telegram rows are posts in the storage channel;
-- bunny / s3 content still lives in lessons.video_id / ebooks.file_key.
CREATE TABLE media_files (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider                TEXT NOT NULL CHECK (provider IN ('bunny', 's3', 'telegram')),
    telegram_file_id        TEXT,
    telegram_file_unique_id TEXT,
    telegram_message_id     BIGINT,
    mime_type               TEXT,
    size_bytes              BIGINT CHECK (size_bytes >= 0),
    file_name               TEXT,
    duration_seconds        INT CHECK (duration_seconds >= 0),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (provider <> 'telegram' OR (telegram_file_id IS NOT NULL AND telegram_file_unique_id IS NOT NULL))
);
-- One row per Telegram file, so re-importing a channel post is a no-op.
CREATE UNIQUE INDEX media_files_telegram_unique_idx ON media_files (telegram_file_unique_id)
    WHERE telegram_file_unique_id IS NOT NULL;
CREATE INDEX media_files_created_idx ON media_files (created_at DESC);

-- When set, the media wins over video_id / file_key.
ALTER TABLE lessons ADD COLUMN media_id UUID REFERENCES media_files (id) ON DELETE SET NULL;
ALTER TABLE ebooks  ADD COLUMN media_id UUID REFERENCES media_files (id) ON DELETE SET NULL;
