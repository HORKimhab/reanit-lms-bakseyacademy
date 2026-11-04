import { redirect } from '@sveltejs/kit';
import { adminCall, cents, checked, text } from '$lib/server/admin';
import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { AdminBundle } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url, params }) => {
	const token = requireToken(locals, url.pathname);
	return { bundle: await api<AdminBundle>(fetch, `/api/admin/bundles/${encodeURIComponent(params.id)}`, { token }) };
};

const base = (id: string) => `/api/admin/bundles/${encodeURIComponent(id)}`;
const itemPath = (id: string, f: FormData) =>
	`${base(id)}/${text(f, 'kind') === 'ebooks' ? 'ebooks' : 'courses'}/${encodeURIComponent(text(f, 'item'))}`;

export const actions: Actions = {
	save: async (event) => {
		const f = await event.request.formData();
		const body = {
			slug: text(f, 'slug'),
			title: text(f, 'title'),
			summary: text(f, 'summary') || null,
			description: text(f, 'description'),
			cover_url: text(f, 'cover_url') || null,
			price_cents: cents(f, 'price') ?? -1,
			sale_price_cents: cents(f, 'sale_price'),
			published: checked(f, 'published')
		};
		const res = await adminCall(event, base(event.params.id), { method: 'PUT', body }, { id: 'fields' });
		return res.ok ? { id: 'fields', saved: true } : res.failure;
	},
	addItem: async (event) => {
		const f = await event.request.formData();
		const id = `add-${text(f, 'kind')}`;
		const res = await adminCall(event, itemPath(event.params.id, f), { method: 'PUT' }, { id });
		return res.ok ? { id, saved: true } : res.failure;
	},
	removeItem: async (event) => {
		const f = await event.request.formData();
		const res = await adminCall(event, itemPath(event.params.id, f), { method: 'DELETE' }, { id: text(f, 'item') });
		return res.ok ? { id: text(f, 'item'), saved: true } : res.failure;
	},
	delete: async (event) => {
		const res = await adminCall(event, base(event.params.id), { method: 'DELETE' }, { id: 'fields' });
		if (!res.ok) return res.failure;
		redirect(303, '/admin/bundles');
	}
};
