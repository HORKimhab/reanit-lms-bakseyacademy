import { redirect } from '@sveltejs/kit';
import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { CourseDetail } from '$lib/types';
import type { PageServerLoad } from './$types';

// "Continue learning": open the course's first lesson; non-owners go back to the course page.
export const load: PageServerLoad = async ({ fetch, params, locals, url }) => {
	const token = requireToken(locals, url.pathname);
	const course = await api<CourseDetail>(fetch, `/api/courses/${encodeURIComponent(params.slug)}`, { token });
	const first = course.sections.flatMap((s) => s.lessons)[0];
	if (!course.owned || !first) redirect(303, `/courses/${params.slug}`);
	redirect(303, `/courses/${params.slug}/learn/${first.id}`);
};
