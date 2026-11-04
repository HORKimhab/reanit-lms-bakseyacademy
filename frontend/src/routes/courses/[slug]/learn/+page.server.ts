import { redirect } from '@sveltejs/kit';
import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import { resumeLesson } from '$lib/resume';
import type { CourseDetail } from '$lib/types';
import type { PageServerLoad } from './$types';

// Non-owners (and courses without lessons) go back to the course page.
export const load: PageServerLoad = async ({ fetch, params, locals, url }) => {
	const token = requireToken(locals, url.pathname);
	const course = await api<CourseDetail>(fetch, `/api/courses/${encodeURIComponent(params.slug)}`, { token });
	const target = resumeLesson(course);
	if (!course.owned || !target) redirect(303, `/courses/${params.slug}`);
	redirect(303, `/courses/${params.slug}/learn/${target}`);
};
