import { redirect } from '@sveltejs/kit';
import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { CourseDetail } from '$lib/types';
import type { PageServerLoad } from './$types';

// "Continue learning": open the first lesson not yet completed (or the first one when all are);
// non-owners go back to the course page.
export const load: PageServerLoad = async ({ fetch, params, locals, url }) => {
	const token = requireToken(locals, url.pathname);
	const course = await api<CourseDetail>(fetch, `/api/courses/${encodeURIComponent(params.slug)}`, { token });
	const lessons = course.sections.flatMap((s) => s.lessons);
	const done = new Set(course.completed_lesson_ids);
	const target = lessons.find((l) => !done.has(l.id)) ?? lessons[0];
	if (!course.owned || !target) redirect(303, `/courses/${params.slug}`);
	redirect(303, `/courses/${params.slug}/learn/${target.id}`);
};
