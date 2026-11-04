import { m } from '$lib/paraglide/messages.js';
import { error, fail, redirect } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { CourseDetail, LessonPlayback, LessonQuestions } from '$lib/types';
import type { Action, Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params, locals, url }) => {
	const slug = encodeURIComponent(params.slug);
	const token = locals.token;
	const lessonPath = `/api/courses/${slug}/lessons/${encodeURIComponent(params.lessonId)}`;
	const [course, lesson, questions] = await Promise.all([
		api<CourseDetail>(fetch, `/api/courses/${slug}`, { token }),
		apiResult<LessonPlayback>(fetch, lessonPath, { token }),
		apiResult<LessonQuestions>(fetch, `${lessonPath}/questions`, { token })
	]);
	if (!lesson.ok) {
		if (lesson.status === 401) redirect(303, `/auth?redirectTo=${encodeURIComponent(url.pathname)}`);
		if (lesson.status === 403) redirect(303, `/courses/${params.slug}`);
		if (lesson.status === 404 || lesson.status === 400) error(404, m.error_not_found());
		error(502, m.error_api_failed());
	}
	return { course, lesson: lesson.data, qa: questions.ok ? questions.data : null };
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

const body = async (request: Request) => String((await request.formData()).get('body') ?? '').trim();
const formId = async (request: Request) => encodeURIComponent(String((await request.formData()).get('id') ?? ''));

/** Q&A calls: `form` tells the page which form the result belongs to. */
async function qa(event: Parameters<Action>[0], form: string, path: string, method: 'POST' | 'DELETE', payload?: unknown) {
	const token = requireToken(event.locals, event.url.pathname);
	const res = await apiResult(event.fetch, path, { method, token, body: payload });
	if (!res.ok) return fail(res.status, { qaForm: form, code: res.code });
	return { qaForm: form, saved: true };
}

export const actions: Actions = {
	complete: (event) => setCompleted(event, true),
	uncomplete: (event) => setCompleted(event, false),
	ask: async (event) => {
		const { params, request } = event;
		const path = `/api/courses/${encodeURIComponent(params.slug)}/lessons/${encodeURIComponent(params.lessonId)}/questions`;
		return qa(event, 'ask', path, 'POST', { body: await body(request) });
	},
	answer: async (event) => {
		const data = await event.request.formData();
		const id = String(data.get('id') ?? '');
		return qa(event, id, `/api/questions/${encodeURIComponent(id)}/answers`, 'POST', { body: String(data.get('body') ?? '').trim() });
	},
	deleteQuestion: async (event) => qa(event, 'delete', `/api/questions/${await formId(event.request)}`, 'DELETE'),
	deleteAnswer: async (event) => qa(event, 'delete', `/api/answers/${await formId(event.request)}`, 'DELETE')
};
