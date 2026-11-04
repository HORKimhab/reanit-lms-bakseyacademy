import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages.js';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { Cart, OrderDetail, PaymentMethod } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

// `?coupon=CODE` previews a discount code; an unusable one shows its error over the plain cart.
export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, '/cart');
	const coupon = url.searchParams.get('coupon')?.trim() ?? '';
	const [preview, methods] = await Promise.all([
		coupon ? apiResult<Cart>(fetch, `/api/cart?coupon=${encodeURIComponent(coupon)}`, { token }) : null,
		api<PaymentMethod[]>(fetch, '/api/payment-methods')
	]);
	if (preview?.ok) return { cart: preview.data, methods, coupon, couponError: null };
	if (preview && preview.status !== 422) error(preview.status === 503 ? 503 : 502, m.error_api_failed());
	const cart = await api<Cart>(fetch, '/api/cart', { token });
	return { cart, methods, coupon, couponError: preview ? preview.code : null };
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
		const data = await request.formData();
		const method = data.get('payment_method');
		const coupon = String(data.get('coupon_code') ?? '').trim();
		const res = await apiResult<OrderDetail>(fetch, '/api/orders', {
			method: 'POST',
			token,
			body: { payment_method: method ? String(method) : null, coupon_code: coupon || null }
		});
		if (!res.ok) return fail(res.status, { code: res.code });
		redirect(303, `/orders/${res.data.id}`);
	}
};
