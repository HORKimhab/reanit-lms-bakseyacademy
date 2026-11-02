import { adminCall } from '$lib/server/admin';
import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { AdminCourseRow, Paged } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const params = new URLSearchParams({ per_page: '20' });
	for (const key of ['q', 'page']) {
		const v = url.searchParams.get(key);
		if (v) params.set(key, v);
	}
	const courses = await api<Paged<AdminCourseRow>>(fetch, `/api/admin/courses?${params}`, { token });
	return { courses, q: url.searchParams.get('q') ?? '' };
};

export const actions: Actions = {
	publish: async (event) => {
		const data = await event.request.formData();
		const id = String(data.get('id') ?? '');
		const res = await adminCall(
			event,
			`/api/admin/courses/${encodeURIComponent(id)}/publish`,
			{ method: 'POST', body: { published: data.get('published') === 'true' } },
			{ id }
		);
		return res.ok ? { id } : res.failure;
	}
};
