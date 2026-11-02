import { error, redirect } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import type { CourseDetail, LessonPlayback } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params, locals, url }) => {
	const slug = encodeURIComponent(params.slug);
	const token = locals.token;
	const [course, lesson] = await Promise.all([
		api<CourseDetail>(fetch, `/api/courses/${slug}`, { token }),
		apiResult<LessonPlayback>(fetch, `/api/courses/${slug}/lessons/${encodeURIComponent(params.lessonId)}`, { token })
	]);
	if (!lesson.ok) {
		if (lesson.status === 401) redirect(303, `/auth?redirectTo=${encodeURIComponent(url.pathname)}`);
		if (lesson.status === 403) redirect(303, `/courses/${params.slug}`);
		if (lesson.status === 404 || lesson.status === 400) error(404, 'រកមិនឃើញ');
		error(502, 'មានបញ្ហាពីម៉ាស៊ីនមេ');
	}
	return { course, lesson: lesson.data };
};
