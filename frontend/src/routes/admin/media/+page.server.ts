import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { AdminMedia, Paged } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const params = new URLSearchParams({ per_page: '30' });
	for (const key of ['q', 'page']) {
		const v = url.searchParams.get(key);
		if (v) params.set(key, v);
	}
	return { media: await api<Paged<AdminMedia>>(fetch, `/api/admin/media?${params}`, { token }), q: url.searchParams.get('q') ?? '' };
};
