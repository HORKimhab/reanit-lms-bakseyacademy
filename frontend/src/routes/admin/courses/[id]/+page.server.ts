import { redirect } from '@sveltejs/kit';
import { adminCall, checked, mediaPickerPath, seconds, text } from '$lib/server/admin';
import { api } from '$lib/server/api';
import { courseBody } from '$lib/server/catalog-forms';
import { requireToken } from '$lib/server/guard';
import type { AdminCategory, AdminCourse, AdminInstructor, AdminMedia, Paged } from '$lib/types';
import type { Action, Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url, params }) => {
	const token = requireToken(locals, url.pathname);
	const [course, instructors, categories, media] = await Promise.all([
		api<AdminCourse>(fetch, `/api/admin/courses/${encodeURIComponent(params.id)}`, { token }),
		api<AdminInstructor[]>(fetch, '/api/admin/instructors', { token }),
		api<AdminCategory[]>(fetch, '/api/admin/categories', { token }),
		api<Paged<AdminMedia>>(fetch, mediaPickerPath(url, 'video'), { token })
	]);
	return { course, instructors, categories, videos: media.items, mediaq: url.searchParams.get('mediaq') ?? '' };
};

const lessonBody = (d: FormData) => ({
	title: text(d, 'title'),
	video_id: text(d, 'video_id'),
	media_id: text(d, 'media_id') || null,
	duration_seconds: seconds(d, 'duration'),
	is_preview: checked(d, 'is_preview')
});

/** Curriculum action: `path(id)` → admin endpoint, `body(form)` → JSON body (none for DELETE). */
function curriculum(
	method: 'POST' | 'PUT' | 'DELETE',
	path: (form: FormData, courseId: string) => string,
	body?: (form: FormData) => unknown
): Action {
	return async (event) => {
		const form = await event.request.formData();
		const res = await adminCall(event, path(form, event.params.id), { method, body: body?.(form) });
		return res.ok ? { saved: true } : res.failure;
	};
}

const id = (form: FormData) => encodeURIComponent(text(form, 'id'));

export const actions: Actions = {
	save: async (event) => {
		const body = courseBody(await event.request.formData());
		const res = await adminCall(event, `/api/admin/courses/${encodeURIComponent(event.params.id)}`, { method: 'PUT', body });
		return res.ok ? { saved: true } : res.failure;
	},
	delete: async (event) => {
		const res = await adminCall(event, `/api/admin/courses/${encodeURIComponent(event.params.id)}`, { method: 'DELETE' });
		if (!res.ok) return res.failure;
		redirect(303, '/admin/courses');
	},
	addSection: curriculum('POST', (_, courseId) => `/api/admin/courses/${encodeURIComponent(courseId)}/sections`, (f) => ({
		title: text(f, 'title')
	})),
	renameSection: curriculum('PUT', (f) => `/api/admin/sections/${id(f)}`, (f) => ({ title: text(f, 'title') })),
	deleteSection: curriculum('DELETE', (f) => `/api/admin/sections/${id(f)}`),
	moveSection: curriculum('POST', (f) => `/api/admin/sections/${id(f)}/move`, (f) => ({ direction: text(f, 'direction') })),
	addLesson: curriculum('POST', (f) => `/api/admin/sections/${id(f)}/lessons`, lessonBody),
	updateLesson: curriculum('PUT', (f) => `/api/admin/lessons/${id(f)}`, lessonBody),
	deleteLesson: curriculum('DELETE', (f) => `/api/admin/lessons/${id(f)}`),
	moveLesson: curriculum('POST', (f) => `/api/admin/lessons/${id(f)}/move`, (f) => ({ direction: text(f, 'direction') }))
};
