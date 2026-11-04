-- Bundles: several courses / ebooks sold together at one price. Buying one grants every item in it.
CREATE TABLE bundles (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug             TEXT NOT NULL UNIQUE,
    title            TEXT NOT NULL,
    summary          TEXT,
    description      TEXT NOT NULL DEFAULT '',
    cover_url        TEXT,
    price_cents      INT  NOT NULL CHECK (price_cents >= 0),
    sale_price_cents INT  CHECK (sale_price_cents >= 0 AND sale_price_cents < price_cents),
    published        BOOLEAN NOT NULL DEFAULT false,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE bundle_items (
    bundle_id UUID NOT NULL REFERENCES bundles (id) ON DELETE CASCADE,
    course_id UUID REFERENCES courses (id) ON DELETE CASCADE,
    ebook_id  UUID REFERENCES ebooks (id) ON DELETE CASCADE,
    position  INT  NOT NULL DEFAULT 0,
    CHECK ((course_id IS NULL) <> (ebook_id IS NULL))
);
CREATE UNIQUE INDEX bundle_items_course_idx ON bundle_items (bundle_id, course_id) WHERE course_id IS NOT NULL;
CREATE UNIQUE INDEX bundle_items_ebook_idx  ON bundle_items (bundle_id, ebook_id)  WHERE ebook_id IS NOT NULL;

-- Carts and orders may hold a bundle as one line.
ALTER TABLE cart_items ADD COLUMN bundle_id UUID REFERENCES bundles (id) ON DELETE CASCADE;
ALTER TABLE cart_items DROP CONSTRAINT cart_items_check;
ALTER TABLE cart_items ADD CONSTRAINT cart_items_one_item
    CHECK (num_nonnulls(course_id, ebook_id, bundle_id) = 1);
CREATE UNIQUE INDEX cart_items_user_bundle_idx ON cart_items (user_id, bundle_id) WHERE bundle_id IS NOT NULL;

ALTER TABLE order_items ADD COLUMN bundle_id UUID REFERENCES bundles (id) ON DELETE SET NULL;
ALTER TABLE order_items DROP CONSTRAINT order_items_kind_check;
ALTER TABLE order_items ADD CONSTRAINT order_items_kind_check CHECK (kind IN ('course', 'ebook', 'bundle'));
