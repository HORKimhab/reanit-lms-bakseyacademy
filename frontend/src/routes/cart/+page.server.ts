import { fail, redirect } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { Cart, OrderDetail, PaymentMethod } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
	const token = requireToken(locals, '/cart');
	const [cart, methods] = await Promise.all([
		api<Cart>(fetch, '/api/cart', { token }),
		api<PaymentMethod[]>(fetch, '/api/payment-methods')
	]);
	return { cart, methods };
};

export const actions: Actions = {
	remove: async ({ locals, fetch, request }) => {
		const token = requireToken(locals, '/cart');
		const id = String((await request.formData()).get('id') ?? '');
		const res = await apiResult(fetch, `/api/cart/${encodeURIComponent(id)}`, { method: 'DELETE', token });
		if (!res.ok) return fail(res.status, { code: res.code });
	},

	checkout: async ({ locals, fetch, request }) => {
		const token = requireToken(locals, '/cart');
		const method = (await request.formData()).get('payment_method');
		const res = await apiResult<OrderDetail>(fetch, '/api/orders', {
			method: 'POST',
			token,
			body: { payment_method: method ? String(method) : null }
		});
		if (!res.ok) return fail(res.status, { code: res.code });
		redirect(303, `/orders/${res.data.id}`);
	}
};
