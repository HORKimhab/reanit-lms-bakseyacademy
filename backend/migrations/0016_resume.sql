-- Where each learner left off, so "Continue learning" resumes there.
ALTER TABLE enrollments ADD COLUMN last_lesson_id UUID REFERENCES lessons (id) ON DELETE SET NULL;
ALTER TABLE enrollments ADD COLUMN last_viewed_at TIMESTAMPTZ;
