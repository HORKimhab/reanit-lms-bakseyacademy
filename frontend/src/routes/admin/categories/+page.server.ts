import { api } from '$lib/server/api';
import { optInt, text } from '$lib/server/admin';
import { crudActions } from '$lib/server/admin-crud';
import { requireToken } from '$lib/server/guard';
import type { AdminCategory } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	return { categories: await api<AdminCategory[]>(fetch, '/api/admin/categories', { token }) };
};

export const actions: Actions = crudActions('categories', (f) => ({
	slug: text(f, 'slug'),
	name_km: text(f, 'name_km'),
	name_en: text(f, 'name_en'),
	position: optInt(f, 'position') ?? 0
}));
