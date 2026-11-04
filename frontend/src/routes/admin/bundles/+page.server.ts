import { redirect } from '@sveltejs/kit';
import { adminCall, cents, text } from '$lib/server/admin';
import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { AdminBundle, AdminBundleRow, Paged } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const params = new URLSearchParams({ per_page: '24' });
	for (const key of ['q', 'page']) {
		const v = url.searchParams.get(key);
		if (v) params.set(key, v);
	}
	return { bundles: await api<Paged<AdminBundleRow>>(fetch, `/api/admin/bundles?${params}`, { token }), q: url.searchParams.get('q') ?? '' };
};

export const actions: Actions = {
	/** Creates a draft from slug + title + price; the rest is edited on its page. */
	create: async (event) => {
		const f = await event.request.formData();
		const body = { slug: text(f, 'slug'), title: text(f, 'title'), price_cents: cents(f, 'price') ?? -1, published: false };
		const res = await adminCall<AdminBundle>(event, '/api/admin/bundles', { method: 'POST', body }, { id: 'new' });
		if (!res.ok) return res.failure;
		redirect(303, `/admin/bundles/${res.data.id}`);
	}
};
