-- Lessons a learner has marked complete.
CREATE TABLE lesson_progress (
    user_id      UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    lesson_id    UUID NOT NULL REFERENCES lessons (id) ON DELETE CASCADE,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, lesson_id)
);
