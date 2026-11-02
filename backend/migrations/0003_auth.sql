-- Phone-number accounts (+855…, E.164) with optional email, and login sessions.
ALTER TABLE users ALTER COLUMN email DROP NOT NULL;
ALTER TABLE users ADD COLUMN phone TEXT;
ALTER TABLE users ADD CONSTRAINT users_phone_e164 CHECK (phone ~ '^\+[1-9][0-9]{7,14}$');
ALTER TABLE users ADD CONSTRAINT users_phone_or_email CHECK (phone IS NOT NULL OR email IS NOT NULL);
CREATE UNIQUE INDEX users_phone_idx ON users (phone);

-- Only a SHA-256 of the session token is stored; the raw token lives in the client's cookie.
CREATE TABLE sessions (
    token_hash BYTEA PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX sessions_user_idx ON sessions (user_id);
