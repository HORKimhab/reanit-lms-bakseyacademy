-- Discount codes applied at checkout: a percentage or a fixed amount off the order.
CREATE TABLE coupons (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code             TEXT NOT NULL CHECK (code ~ '^[A-Z0-9_-]{3,32}$'),   -- stored upper-case
    percent_off      INT  CHECK (percent_off BETWEEN 1 AND 100),
    amount_off_cents INT  CHECK (amount_off_cents > 0),
    expires_at       TIMESTAMPTZ,       -- NULL = never
    max_uses         INT  CHECK (max_uses > 0),  -- NULL = unlimited; every non-cancelled order counts
    active           BOOLEAN NOT NULL DEFAULT true,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((percent_off IS NULL) <> (amount_off_cents IS NULL))
);
CREATE UNIQUE INDEX coupons_code_idx ON coupons (code);

-- Discount snapshot: total_cents = sum(order_items.price_cents) - discount_cents.
ALTER TABLE orders ADD COLUMN coupon_id      UUID REFERENCES coupons (id) ON DELETE SET NULL;
ALTER TABLE orders ADD COLUMN coupon_code    TEXT;
ALTER TABLE orders ADD COLUMN discount_cents INT NOT NULL DEFAULT 0 CHECK (discount_cents >= 0);
CREATE INDEX orders_coupon_idx ON orders (coupon_id) WHERE coupon_id IS NOT NULL;
