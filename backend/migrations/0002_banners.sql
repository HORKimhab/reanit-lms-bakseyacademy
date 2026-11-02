-- Homepage slideshow banners.
CREATE TABLE banners (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title      TEXT NOT NULL,
    subtitle   TEXT,
    image_url  TEXT,               -- NULL renders a brand gradient behind the text
    link_url   TEXT,               -- site-relative path or absolute URL
    position   INT  NOT NULL DEFAULT 0,
    published  BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX banners_published_position_idx ON banners (position) WHERE published;
