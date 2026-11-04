-- Course completion certificates, issued when a learner completes every lesson. Name and title are
-- snapshots at issue time; the id doubles as the public verification link.
CREATE TABLE certificates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    course_id       UUID NOT NULL REFERENCES courses (id) ON DELETE CASCADE,
    full_name       TEXT NOT NULL,
    course_title    TEXT NOT NULL,
    instructor_name TEXT NOT NULL,
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, course_id)
);

-- Learners who had already finished a course get theirs now.
INSERT INTO certificates (user_id, course_id, full_name, course_title, instructor_name)
SELECT en.user_id, c.id, u.full_name, c.title, i.name
FROM enrollments en
JOIN users u ON u.id = en.user_id
JOIN courses c ON c.id = en.course_id
JOIN instructors i ON i.id = c.instructor_id
WHERE EXISTS (SELECT 1 FROM lessons l JOIN course_sections s ON s.id = l.section_id WHERE s.course_id = c.id)
  AND NOT EXISTS (
      SELECT 1 FROM lessons l JOIN course_sections s ON s.id = l.section_id
      WHERE s.course_id = c.id
        AND NOT EXISTS (SELECT 1 FROM lesson_progress p WHERE p.lesson_id = l.id AND p.user_id = en.user_id));
