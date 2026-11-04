import { fail, redirect } from '@sveltejs/kit';
import { apiResult } from '$lib/server/api';
import { clearSession, safeRedirect, setSession } from '$lib/server/session';
import type { AuthOptions, AuthResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, url, fetch }) => {
	const redirectTo = safeRedirect(url.searchParams.get('redirectTo'));
	if (locals.user) redirect(303, redirectTo);
	const options = await apiResult<AuthOptions>(fetch, '/api/auth/options');
	return {
		tab: url.searchParams.get('tab') === 'register' ? 'register' : 'login',
		redirectTo,
		options: options.ok ? options.data : { google: false, password_reset: false },
		// Set by the Google callback (`?error=google_failed`, …).
		oauthError: url.searchParams.get('error')
	};
};

const field = (data: FormData, name: string) => String(data.get(name) ?? '');

export const actions: Actions = {
	login: async ({ request, fetch, cookies, url }) => {
		const data = await request.formData();
		const phone = field(data, 'phone');
		const res = await apiResult<AuthResponse>(fetch, '/api/auth/login', {
			method: 'POST',
			body: { phone, password: field(data, 'password') }
		});
		if (!res.ok) return fail(res.status, { form: 'login' as const, code: res.code, phone });
		setSession(cookies, res.data);
		redirect(303, safeRedirect(url.searchParams.get('redirectTo')));
	},

	register: async ({ request, fetch, cookies, url }) => {
		const data = await request.formData();
		const full_name = field(data, 'full_name');
		const phone = field(data, 'phone');
		const password = field(data, 'password');
		if (password !== field(data, 'password_confirm')) {
			return fail(422, { form: 'register' as const, code: 'password_mismatch', phone, full_name });
		}
		const res = await apiResult<AuthResponse>(fetch, '/api/auth/register', {
			method: 'POST',
			body: { full_name, phone, password }
		});
		if (!res.ok) return fail(res.status, { form: 'register' as const, code: res.code, phone, full_name });
		setSession(cookies, res.data);
		redirect(303, safeRedirect(url.searchParams.get('redirectTo')));
	},

	logout: async ({ fetch, cookies, locals }) => {
		if (locals.token) await apiResult(fetch, '/api/auth/logout', { method: 'POST', token: locals.token });
		clearSession(cookies);
		redirect(303, '/');
	}
};
