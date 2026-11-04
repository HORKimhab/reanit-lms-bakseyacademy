import { fail } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import QRCode from 'qrcode';
import type { KhqrPayment, OrderDetail, PaywayCheckout } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

const path = (id: string) => `/api/orders/${encodeURIComponent(id)}`;

const PAYABLE = ['pending_payment', 'awaiting_approval', 'rejected'];

/**
 * For automatically confirmed methods the API is asked to check with the provider on every load
 * (the page reloads itself every few seconds while waiting), and Bakong orders get their KHQR.
 */
export const load: PageServerLoad = async ({ locals, fetch, params }) => {
	const token = requireToken(locals, `/orders/${params.id}`);
	let order = await api<OrderDetail>(fetch, path(params.id), { token });
	let khqr: (KhqrPayment & { svg: string }) | null = null;
	if (PAYABLE.includes(order.status) && order.payment_method?.automatic) {
		const checked = await apiResult<OrderDetail>(fetch, `${path(params.id)}/check-payment`, { method: 'POST', token });
		if (checked.ok) order = checked.data;
		if (PAYABLE.includes(order.status) && order.payment_method?.code === 'bakong_khqr') {
			const res = await apiResult<KhqrPayment>(fetch, `${path(params.id)}/khqr`, { method: 'POST', token });
			if (res.ok) {
				const svg = await QRCode.toString(res.data.qr, { type: 'svg', margin: 1, errorCorrectionLevel: 'M' });
				khqr = { ...res.data, svg };
			}
		}
	}
	return { order, khqr };
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

	/** A signed PayWay checkout; the page posts it to PayWay. */
	payway: async ({ locals, fetch, params }) => {
		const token = requireToken(locals, `/orders/${params.id}`);
		const res = await apiResult<PaywayCheckout>(fetch, `${path(params.id)}/payway`, { method: 'POST', token });
		if (!res.ok) return fail(res.status, { code: res.code, payment_ref: '' });
		return { payway: res.data };
	},

	cancel: async ({ locals, fetch, params }) => {
		const token = requireToken(locals, `/orders/${params.id}`);
		const res = await apiResult(fetch, `${path(params.id)}/cancel`, { method: 'POST', token });
		if (!res.ok) return fail(res.status, { code: res.code, payment_ref: '' });
	}
};
