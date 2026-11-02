import { api } from '$lib/server/api';
import { checked, optInt, text } from '$lib/server/admin';
import { crudActions } from '$lib/server/admin-crud';
import { requireToken } from '$lib/server/guard';
import type { AdminBanner } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	return { banners: await api<AdminBanner[]>(fetch, '/api/admin/banners', { token }) };
};

export const actions: Actions = crudActions('banners', (f) => ({
	title: text(f, 'title'),
	subtitle: text(f, 'subtitle'),
	image_url: text(f, 'image_url'),
	link_url: text(f, 'link_url'),
	position: optInt(f, 'position') ?? 0,
	published: checked(f, 'published')
}));
