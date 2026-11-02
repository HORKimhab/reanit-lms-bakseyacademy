import { redirect } from '@sveltejs/kit';
import { adminCall } from '$lib/server/admin';
import { api } from '$lib/server/api';
import { courseBody } from '$lib/server/catalog-forms';
import { requireToken } from '$lib/server/guard';
import type { AdminCategory, AdminCourse, AdminInstructor } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const [instructors, categories] = await Promise.all([
		api<AdminInstructor[]>(fetch, '/api/admin/instructors', { token }),
		api<AdminCategory[]>(fetch, '/api/admin/categories', { token })
	]);
	return { instructors, categories };
};

export const actions: Actions = {
	default: async (event) => {
		const body = courseBody(await event.request.formData());
		const res = await adminCall<AdminCourse>(event, '/api/admin/courses', { method: 'POST', body });
		if (!res.ok) return res.failure;
		redirect(303, `/admin/courses/${res.data.id}`);
	}
};
