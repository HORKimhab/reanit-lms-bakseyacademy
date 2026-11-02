import { redirect } from '@sveltejs/kit';
import { adminCall } from '$lib/server/admin';
import { api } from '$lib/server/api';
import { ebookBody } from '$lib/server/catalog-forms';
import { requireToken } from '$lib/server/guard';
import type { AdminCategory, AdminEbook, AdminMedia, Paged } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url, params }) => {
	const token = requireToken(locals, url.pathname);
	const [ebook, categories, media] = await Promise.all([
		api<AdminEbook>(fetch, `/api/admin/ebooks/${encodeURIComponent(params.id)}`, { token }),
		api<AdminCategory[]>(fetch, '/api/admin/categories', { token }),
		api<Paged<AdminMedia>>(fetch, '/api/admin/media?kind=document&per_page=200', { token })
	]);
	return { ebook, categories, documents: media.items };
};

export const actions: Actions = {
	save: async (event) => {
		const body = ebookBody(await event.request.formData());
		const res = await adminCall(event, `/api/admin/ebooks/${encodeURIComponent(event.params.id)}`, { method: 'PUT', body });
		return res.ok ? { saved: true } : res.failure;
	},
	delete: async (event) => {
		const res = await adminCall(event, `/api/admin/ebooks/${encodeURIComponent(event.params.id)}`, { method: 'DELETE' });
		if (!res.ok) return res.failure;
		redirect(303, '/admin/ebooks');
	}
};
