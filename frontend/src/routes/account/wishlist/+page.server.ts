import { fail, redirect } from '@sveltejs/kit';
import { api, apiResult } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import { safeRedirect } from '$lib/server/session';
import type { Wishlist } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
	const token = requireToken(locals, '/account/wishlist');
	return { list: await api<Wishlist>(fetch, '/api/me/wishlist', { token }) };
};

export const actions: Actions = {
	/** Saves or unsaves (`saved` = currently saved) a course / ebook, then goes back to `redirectTo`. */
	toggle: async ({ locals, fetch, request }) => {
		const data = await request.formData();
		const back = safeRedirect(String(data.get('redirectTo') ?? ''), '/account/wishlist');
		const token = requireToken(locals, back);
		const kind = data.get('kind') === 'ebook' ? 'ebooks' : 'courses';
		const id = encodeURIComponent(String(data.get('id') ?? ''));
		const res = await apiResult(fetch, `/api/me/wishlist/${kind}/${id}`, { method: data.get('saved') ? 'DELETE' : 'PUT', token });
		if (!res.ok) return fail(res.status, { code: res.code });
		redirect(303, back);
	}
};
