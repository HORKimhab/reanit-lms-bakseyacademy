import { fail, redirect } from '@sveltejs/kit';
import { apiResult } from '$lib/server/api';
import { setSession } from '$lib/server/session';
import type { AuthResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ locals }) => {
	if (locals.user) redirect(303, '/account');
};

const field = (data: FormData, name: string) => String(data.get(name) ?? '').trim();

export const actions: Actions = {
	/** Step 1: text a code (the API answers the same whether or not the number has an account). */
	request: async ({ request, fetch }) => {
		const phone = field(await request.formData(), 'phone');
		const res = await apiResult(fetch, '/api/auth/password-reset', { method: 'POST', body: { phone } });
		if (!res.ok) return fail(res.status, { step: 'request' as const, code: res.code, phone });
		return { step: 'confirm' as const, phone };
	},

	/** Step 2: code + new password → signed in. */
	confirm: async ({ request, fetch, cookies }) => {
		const data = await request.formData();
		const phone = field(data, 'phone');
		const new_password = String(data.get('new_password') ?? '');
		if (new_password !== String(data.get('new_password_confirm') ?? '')) {
			return fail(422, { step: 'confirm' as const, code: 'password_mismatch', phone });
		}
		const res = await apiResult<AuthResponse>(fetch, '/api/auth/password-reset/confirm', {
			method: 'POST',
			body: { phone, code: field(data, 'code'), new_password }
		});
		if (!res.ok) return fail(res.status, { step: 'confirm' as const, code: res.code, phone });
		setSession(cookies, res.data);
		redirect(303, '/account');
	}
};
