-- Google sign-in: the stable Google account id (`sub` claim). Linked on sign-up with Google or
-- from /account while signed in; never by matching email.
ALTER TABLE users ADD COLUMN google_sub TEXT UNIQUE;
