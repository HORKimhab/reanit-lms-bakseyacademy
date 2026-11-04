import type { CourseDetail } from '$lib/types';

/**
 * "Continue learning": resume at the lesson opened last; once that one is complete, move on to the
 * next lesson not yet completed after it. Without history, the first incomplete lesson (or the first
 * lesson when all are done); `null` for a course without lessons.
 */
export function resumeLesson(course: Pick<CourseDetail, 'sections' | 'completed_lesson_ids' | 'last_lesson_id'>): string | null {
	const lessons = course.sections.flatMap((s) => s.lessons);
	const done = new Set(course.completed_lesson_ids);
	const last = lessons.findIndex((l) => l.id === course.last_lesson_id);
	if (last >= 0) {
		if (!done.has(lessons[last].id)) return lessons[last].id;
		const after = lessons.slice(last + 1).find((l) => !done.has(l.id));
		if (after) return after.id;
	}
	return (lessons.find((l) => !done.has(l.id)) ?? lessons[0])?.id ?? null;
}
