import { api } from '$lib/server/api';
import { adminCall, text } from '$lib/server/admin';
import { requireToken } from '$lib/server/guard';
import type { AdminUserDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url, params }) => {
	const token = requireToken(locals, url.pathname);
	return { u: await api<AdminUserDetail>(fetch, `/api/admin/users/${encodeURIComponent(params.id)}`, { token }) };
};

/** `kind` = courses | ebooks, `item` = id. */
const itemPath = (userId: string, form: FormData) =>
	`/api/admin/users/${encodeURIComponent(userId)}/${text(form, 'kind') === 'ebooks' ? 'ebooks' : 'courses'}/${encodeURIComponent(text(form, 'item'))}`;

export const actions: Actions = {
	role: async (event) => {
		const form = await event.request.formData();
		const res = await adminCall(event, `/api/admin/users/${encodeURIComponent(event.params.id)}/role`, { method: 'PUT', body: { role: text(form, 'role') } }, { id: 'role' });
		return res.ok ? { id: 'role', saved: true } : res.failure;
	},
	grant: async (event) => {
		const form = await event.request.formData();
		const id = `grant-${text(form, 'kind')}`;
		const res = await adminCall(event, itemPath(event.params.id, form), { method: 'PUT' }, { id });
		return res.ok ? { id, saved: true } : res.failure;
	},
	revoke: async (event) => {
		const form = await event.request.formData();
		const res = await adminCall(event, itemPath(event.params.id, form), { method: 'DELETE' }, { id: text(form, 'item') });
		return res.ok ? { id: text(form, 'item'), saved: true } : res.failure;
	}
};
