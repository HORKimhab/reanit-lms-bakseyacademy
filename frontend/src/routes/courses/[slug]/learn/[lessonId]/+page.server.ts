import { error, fail, redirect } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { CourseDetail, LessonPlayback } from '$lib/types';
import type { Action, Actions, PageServerLoad } from './$types';

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

/** Mark complete / not complete. `next` (a lesson id) continues to that lesson afterwards. */
async function setCompleted(event: Parameters<Action>[0], completed: boolean) {
	const { locals, fetch, params, request, url } = event;
	const token = requireToken(locals, url.pathname);
	const path = `/api/courses/${encodeURIComponent(params.slug)}/lessons/${encodeURIComponent(params.lessonId)}/complete`;
	const res = await apiResult(fetch, path, { method: completed ? 'PUT' : 'DELETE', token });
	if (!res.ok) return fail(res.status, { code: res.code });
	const next = (await request.formData()).get('next');
	if (completed && typeof next === 'string' && next) redirect(303, `/courses/${params.slug}/learn/${encodeURIComponent(next)}`);
	return { completed };
}

export const actions: Actions = {
	complete: (event) => setCompleted(event, true),
	uncomplete: (event) => setCompleted(event, false)
};
