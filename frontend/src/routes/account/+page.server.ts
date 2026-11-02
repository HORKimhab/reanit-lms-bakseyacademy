import { fail, redirect } from '@sveltejs/kit';
import { apiResult } from '$lib/server/api';
import type { User } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ locals }) => {
	if (!locals.user) redirect(303, '/auth?redirectTo=/account');
	return { user: locals.user };
};

const field = (data: FormData, name: string) => String(data.get(name) ?? '');

export const actions: Actions = {
	profile: async ({ request, fetch, locals }) => {
		if (!locals.token) redirect(303, '/auth?redirectTo=/account');
		const data = await request.formData();
		const res = await apiResult<User>(fetch, '/api/me', {
			method: 'PATCH',
			token: locals.token,
			body: { full_name: field(data, 'full_name'), email: field(data, 'email') }
		});
		if (!res.ok) return fail(res.status, { form: 'profile' as const, code: res.code });
		locals.user = res.data; // so a no-JS POST re-renders with the new name
		return { form: 'profile' as const, saved: true };
	},

	password: async ({ request, fetch, locals }) => {
		if (!locals.token) redirect(303, '/auth?redirectTo=/account');
		const data = await request.formData();
		const new_password = field(data, 'new_password');
		if (new_password !== field(data, 'new_password_confirm')) {
			return fail(422, { form: 'password' as const, code: 'password_mismatch' });
		}
		const res = await apiResult(fetch, '/api/me/password', {
			method: 'POST',
			token: locals.token,
			body: { current_password: field(data, 'current_password'), new_password }
		});
		if (!res.ok) return fail(res.status, { form: 'password' as const, code: res.code });
		return { form: 'password' as const, saved: true };
	}
};
