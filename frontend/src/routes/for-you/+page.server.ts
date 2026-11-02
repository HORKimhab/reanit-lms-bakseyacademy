import { api } from '$lib/server/api';
import type { Recommendations } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, locals }) => ({
	recs: await api<Recommendations>(fetch, '/api/recommendations', { token: locals.token })
});
