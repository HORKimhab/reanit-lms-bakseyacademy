-- Automatic payment confirmation (Bakong KHQR, ABA PayWay). One row per payment attempt; the
-- provider reference is what we look the payment up by (KHQR md5 / PayWay tran_id).
CREATE TABLE payment_intents (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id     UUID NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
    provider     TEXT NOT NULL CHECK (provider IN ('bakong', 'payway')),
    provider_ref TEXT NOT NULL,
    amount_cents INT  NOT NULL CHECK (amount_cents > 0),
    currency     TEXT NOT NULL DEFAULT 'USD',
    qr           TEXT,                -- the KHQR string (bakong)
    status       TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'paid', 'expired', 'failed')),
    provider_txn TEXT,                -- provider's transaction hash / approval code once paid
    expires_at   TIMESTAMPTZ NOT NULL,
    checked_at   TIMESTAMPTZ,
    paid_at      TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_ref)
);
CREATE INDEX payment_intents_order_idx ON payment_intents (order_id);
CREATE INDEX payment_intents_open_idx ON payment_intents (created_at) WHERE status = 'open';

-- How a paid order was paid: free checkout, admin approval, or a provider confirmation.
ALTER TABLE orders ADD COLUMN paid_via TEXT CHECK (paid_via IN ('free', 'admin', 'bakong', 'payway'));
UPDATE orders SET paid_via = CASE WHEN total_cents = 0 THEN 'free' ELSE 'admin' END WHERE status = 'paid';
