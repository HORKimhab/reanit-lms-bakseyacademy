import { redirect } from '@sveltejs/kit';
import { adminCall, mediaPickerPath } from '$lib/server/admin';
import { api } from '$lib/server/api';
import { ebookBody } from '$lib/server/catalog-forms';
import { requireToken } from '$lib/server/guard';
import type { AdminCategory, AdminEbook, AdminMedia, Paged } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const [categories, media] = await Promise.all([
		api<AdminCategory[]>(fetch, '/api/admin/categories', { token }),
		api<Paged<AdminMedia>>(fetch, mediaPickerPath(url, 'document'), { token })
	]);
	return { categories, documents: media.items, mediaq: url.searchParams.get('mediaq') ?? '' };
};

export const actions: Actions = {
	default: async (event) => {
		const body = ebookBody(await event.request.formData());
		const res = await adminCall<AdminEbook>(event, '/api/admin/ebooks', { method: 'POST', body });
		if (!res.ok) return res.failure;
		redirect(303, `/admin/ebooks/${res.data.id}`);
	}
};
