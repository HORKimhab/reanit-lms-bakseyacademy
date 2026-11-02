-- Commerce: payment methods (static QR for now), cart, orders, and what a paid order grants.

CREATE TABLE payment_methods (
    code           TEXT PRIMARY KEY CHECK (code ~ '^[a-z0-9_]+$'),  -- e.g. bakong_khqr, aba_payway
    name_km        TEXT NOT NULL,
    name_en        TEXT NOT NULL,
    account_name   TEXT NOT NULL DEFAULT '',
    account_number TEXT NOT NULL DEFAULT '',
    qr_image_url   TEXT,               -- static merchant QR the user scans
    position       INT  NOT NULL DEFAULT 0,
    active         BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE cart_items (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    course_id  UUID REFERENCES courses (id) ON DELETE CASCADE,
    ebook_id   UUID REFERENCES ebooks (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((course_id IS NULL) <> (ebook_id IS NULL))
);
CREATE UNIQUE INDEX cart_items_user_course_idx ON cart_items (user_id, course_id) WHERE course_id IS NOT NULL;
CREATE UNIQUE INDEX cart_items_user_ebook_idx  ON cart_items (user_id, ebook_id)  WHERE ebook_id IS NOT NULL;

CREATE TABLE orders (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    status         TEXT NOT NULL DEFAULT 'pending_payment' CHECK (status IN
                     ('pending_payment', 'awaiting_approval', 'paid', 'rejected', 'cancelled')),
    payment_method TEXT REFERENCES payment_methods (code),     -- NULL only for free orders
    total_cents    INT  NOT NULL CHECK (total_cents >= 0),
    payment_ref    TEXT,               -- transaction reference typed by the user
    submitted_at   TIMESTAMPTZ,
    admin_note     TEXT,               -- shown to the user on rejection
    reviewed_by    UUID REFERENCES users (id) ON DELETE SET NULL,
    reviewed_at    TIMESTAMPTZ,
    paid_at        TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (payment_method IS NOT NULL OR total_cents = 0)
);
CREATE INDEX orders_user_created_idx ON orders (user_id, created_at DESC);
CREATE INDEX orders_status_created_idx ON orders (status, created_at);

CREATE TABLE order_items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id    UUID NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
    course_id   UUID REFERENCES courses (id) ON DELETE SET NULL,
    ebook_id    UUID REFERENCES ebooks (id) ON DELETE SET NULL,
    kind        TEXT NOT NULL CHECK (kind IN ('course', 'ebook')),
    title       TEXT NOT NULL,         -- snapshot at checkout
    price_cents INT  NOT NULL CHECK (price_cents >= 0)
);
CREATE INDEX order_items_order_idx ON order_items (order_id);

CREATE TABLE enrollments (
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    course_id  UUID NOT NULL REFERENCES courses (id) ON DELETE CASCADE,
    order_id   UUID REFERENCES orders (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, course_id)
);

CREATE TABLE ebook_purchases (
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    ebook_id   UUID NOT NULL REFERENCES ebooks (id) ON DELETE CASCADE,
    order_id   UUID REFERENCES orders (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, ebook_id)
);
