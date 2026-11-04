-- Lesson Q&A: enrolled learners ask under a lesson; admins and the course's instructor answer.
CREATE TABLE lesson_questions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lesson_id  UUID NOT NULL REFERENCES lessons (id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    body       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX lesson_questions_lesson_idx ON lesson_questions (lesson_id, created_at DESC);
CREATE INDEX lesson_questions_created_idx ON lesson_questions (created_at DESC);

CREATE TABLE lesson_answers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID NOT NULL REFERENCES lesson_questions (id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    body        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX lesson_answers_question_idx ON lesson_answers (question_id, created_at);
