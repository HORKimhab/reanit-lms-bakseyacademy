import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { Library } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
	const token = requireToken(locals, '/account/library');
	return { library: await api<Library>(fetch, '/api/me/library', { token }) };
};
