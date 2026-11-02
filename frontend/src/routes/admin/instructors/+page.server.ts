import { api } from '$lib/server/api';
import { text } from '$lib/server/admin';
import { crudActions } from '$lib/server/admin-crud';
import { requireToken } from '$lib/server/guard';
import type { AdminInstructor } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	return { instructors: await api<AdminInstructor[]>(fetch, '/api/admin/instructors', { token }) };
};

export const actions: Actions = crudActions('instructors', (f) => ({
	name: text(f, 'name'),
	bio: text(f, 'bio'),
	avatar_url: text(f, 'avatar_url')
}));
