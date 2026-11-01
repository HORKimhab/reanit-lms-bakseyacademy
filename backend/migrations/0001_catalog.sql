-- Core catalog schema: users, instructors, categories, courses (+ curriculum), ebooks, reviews.
-- Prices are stored in US cents to avoid floating point.

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL,
    password_hash TEXT,
    full_name     TEXT NOT NULL,
    role          TEXT NOT NULL DEFAULT 'student' CHECK (role IN ('student', 'instructor', 'admin')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX users_email_lower_idx ON users (lower(email));

CREATE TABLE instructors (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID UNIQUE REFERENCES users (id) ON DELETE SET NULL,
    name       TEXT NOT NULL,
    bio        TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE categories (
    id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug     TEXT NOT NULL UNIQUE,
    name_km  TEXT NOT NULL,
    name_en  TEXT NOT NULL,
    position INT  NOT NULL DEFAULT 0
);

CREATE TABLE courses (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug             TEXT NOT NULL UNIQUE,
    title            TEXT NOT NULL,
    summary          TEXT,
    description      TEXT NOT NULL DEFAULT '',
    cover_url        TEXT,
    instructor_id    UUID NOT NULL REFERENCES instructors (id),
    category_id      UUID REFERENCES categories (id) ON DELETE SET NULL,
    price_cents      INT  NOT NULL CHECK (price_cents >= 0),
    sale_price_cents INT  CHECK (sale_price_cents >= 0 AND sale_price_cents < price_cents),
    published        BOOLEAN NOT NULL DEFAULT false,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX courses_published_created_idx ON courses (created_at DESC) WHERE published;
CREATE INDEX courses_category_idx ON courses (category_id);

CREATE TABLE course_sections (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL REFERENCES courses (id) ON DELETE CASCADE,
    title     TEXT NOT NULL,
    position  INT  NOT NULL,
    UNIQUE (course_id, position)
);

CREATE TABLE lessons (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_id       UUID NOT NULL REFERENCES course_sections (id) ON DELETE CASCADE,
    title            TEXT NOT NULL,
    position         INT  NOT NULL,
    video_id         TEXT,               -- id in the video provider (Bunny/Cloudflare Stream)
    duration_seconds INT  NOT NULL DEFAULT 0,
    is_preview       BOOLEAN NOT NULL DEFAULT false,
    UNIQUE (section_id, position)
);

CREATE TABLE ebooks (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug             TEXT NOT NULL UNIQUE,
    title            TEXT NOT NULL,
    author_name      TEXT NOT NULL,
    summary          TEXT,
    description      TEXT NOT NULL DEFAULT '',
    cover_url        TEXT,
    file_key         TEXT,               -- object key in S3/R2; never exposed publicly
    page_count       INT,
    category_id      UUID REFERENCES categories (id) ON DELETE SET NULL,
    price_cents      INT  NOT NULL CHECK (price_cents >= 0),
    sale_price_cents INT  CHECK (sale_price_cents >= 0 AND sale_price_cents < price_cents),
    published        BOOLEAN NOT NULL DEFAULT false,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ebooks_published_created_idx ON ebooks (created_at DESC) WHERE published;

CREATE TABLE reviews (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    course_id  UUID REFERENCES courses (id) ON DELETE CASCADE,
    ebook_id   UUID REFERENCES ebooks (id) ON DELETE CASCADE,
    rating     SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment    TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((course_id IS NULL) <> (ebook_id IS NULL))
);
CREATE UNIQUE INDEX reviews_user_course_idx ON reviews (user_id, course_id) WHERE course_id IS NOT NULL;
CREATE UNIQUE INDEX reviews_user_ebook_idx  ON reviews (user_id, ebook_id)  WHERE ebook_id IS NOT NULL;
CREATE INDEX reviews_course_idx ON reviews (course_id);
CREATE INDEX reviews_ebook_idx  ON reviews (ebook_id);
