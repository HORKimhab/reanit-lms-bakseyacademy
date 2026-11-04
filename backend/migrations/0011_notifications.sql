-- In-app notices for buyers (order approved / rejected), shown under the header bell.
CREATE TABLE notifications (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('order_paid', 'order_rejected')),
    order_id   UUID REFERENCES orders (id) ON DELETE CASCADE,
    note       TEXT,               -- the admin's rejection note
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    read_at    TIMESTAMPTZ
);
CREATE INDEX notifications_user_created_idx ON notifications (user_id, created_at DESC);
