import { api } from '$lib/server/api';
import { adminCall, text } from '$lib/server/admin';
import { requireToken } from '$lib/server/guard';
import type { AdminQuestion, Paged } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const all = url.searchParams.get('all') === '1';
	const params = new URLSearchParams({ unanswered: String(!all), per_page: '20' });
	const page = url.searchParams.get('page');
	if (page) params.set('page', page);
	return { questions: await api<Paged<AdminQuestion>>(fetch, `/api/admin/questions?${params}`, { token }), all };
};

export const actions: Actions = {
	answer: async (event) => {
		const form = await event.request.formData();
		const id = text(form, 'id');
		const res = await adminCall(event, `/api/questions/${encodeURIComponent(id)}/answers`, { method: 'POST', body: { body: text(form, 'body') } }, { id });
		return res.ok ? { id, saved: true } : res.failure;
	}
};
