import { fail } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { AdminOrder, Paged } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	// No ?status= means the approval queue; ?status=all shows everything.
	const status = url.searchParams.get('status') ?? 'awaiting_approval';
	const q = url.searchParams.get('q') ?? '';
	const params = new URLSearchParams();
	if (status !== 'all') params.set('status', status);
	if (q) params.set('q', q);
	const page = url.searchParams.get('page');
	if (page) params.set('page', page);
	params.set('per_page', '20');
	const orders = await api<Paged<AdminOrder>>(fetch, `/api/admin/orders?${params}`, { token });
	return { orders, status, q };
};

export const actions: Actions = {
	approve: async ({ locals, fetch, request, url }) => {
		const token = requireToken(locals, url.pathname);
		const id = String((await request.formData()).get('id') ?? '');
		const res = await apiResult(fetch, `/api/admin/orders/${encodeURIComponent(id)}/approve`, { method: 'POST', token });
		if (!res.ok) return fail(res.status, { id, code: res.code });
		return { id, done: 'approved' as const };
	},

	reject: async ({ locals, fetch, request, url }) => {
		const token = requireToken(locals, url.pathname);
		const data = await request.formData();
		const id = String(data.get('id') ?? '');
		const res = await apiResult(fetch, `/api/admin/orders/${encodeURIComponent(id)}/reject`, {
			method: 'POST',
			token,
			body: { note: String(data.get('note') ?? '') }
		});
		if (!res.ok) return fail(res.status, { id, code: res.code });
		return { id, done: 'rejected' as const };
	}
};
