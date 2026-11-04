import { fail } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { Notifications } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
	const token = requireToken(locals, '/account/notifications');
	return { list: await api<Notifications>(fetch, '/api/me/notifications', { token }) };
};

export const actions: Actions = {
	read: async ({ locals, fetch }) => {
		const token = requireToken(locals, '/account/notifications');
		const res = await apiResult(fetch, '/api/me/notifications/read', { method: 'POST', token });
		if (!res.ok) return fail(res.status, { code: res.code });
		return { read: true };
	}
};
