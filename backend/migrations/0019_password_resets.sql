-- Self-service password reset: a 6-digit code texted to the account's phone. Only an HMAC of the
-- code is stored; a code expires after 10 minutes, dies after 5 wrong guesses, and works once.
CREATE TABLE password_resets (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    code_hash  BYTEA NOT NULL,
    attempts   INT  NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at    TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX password_resets_user_idx ON password_resets (user_id, created_at DESC);
