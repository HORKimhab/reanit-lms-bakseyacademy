-- Items a user saved for later (heart on cards and detail pages).
CREATE TABLE wishlist_items (
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    course_id  UUID REFERENCES courses (id) ON DELETE CASCADE,
    ebook_id   UUID REFERENCES ebooks (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((course_id IS NULL) <> (ebook_id IS NULL))
);
CREATE UNIQUE INDEX wishlist_user_course_idx ON wishlist_items (user_id, course_id) WHERE course_id IS NOT NULL;
CREATE UNIQUE INDEX wishlist_user_ebook_idx  ON wishlist_items (user_id, ebook_id)  WHERE ebook_id IS NOT NULL;
