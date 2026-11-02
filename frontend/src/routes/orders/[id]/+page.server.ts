import { fail } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { OrderDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

const path = (id: string) => `/api/orders/${encodeURIComponent(id)}`;

export const load: PageServerLoad = async ({ locals, fetch, params }) => {
	const token = requireToken(locals, `/orders/${params.id}`);
	return { order: await api<OrderDetail>(fetch, path(params.id), { token }) };
};

export const actions: Actions = {
	submit: async ({ locals, fetch, params, request }) => {
		const token = requireToken(locals, `/orders/${params.id}`);
		const payment_ref = String((await request.formData()).get('payment_ref') ?? '');
		const res = await apiResult(fetch, `${path(params.id)}/submit`, { method: 'POST', token, body: { payment_ref } });
		if (!res.ok) return fail(res.status, { code: res.code, payment_ref });
	},

	/** Payment screenshot → API (which posts it to the admins' Telegram chat). */
	proof: async ({ locals, fetch, params, request }) => {
		const token = requireToken(locals, `/orders/${params.id}`);
		const file = (await request.formData()).get('screenshot');
		if (!(file instanceof File) || file.size === 0) return fail(422, { code: 'missing_file', payment_ref: '' });
		const form = new FormData();
		form.append('size', String(file.size)); // before the file: the API streams it to Telegram
		form.append('file', file, file.name);
		const res = await apiResult(fetch, `${path(params.id)}/proof`, { method: 'POST', token, form });
		if (!res.ok) return fail(res.status, { code: res.code, payment_ref: '' });
		return { proofUploaded: true };
	},

	cancel: async ({ locals, fetch, params }) => {
		const token = requireToken(locals, `/orders/${params.id}`);
		const res = await apiResult(fetch, `${path(params.id)}/cancel`, { method: 'POST', token });
		if (!res.ok) return fail(res.status, { code: res.code, payment_ref: '' });
	}
};
