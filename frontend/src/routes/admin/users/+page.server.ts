import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { AdminUserRow, Paged } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const params = new URLSearchParams({ per_page: '20' });
	for (const key of ['q', 'role', 'page']) {
		const v = url.searchParams.get(key);
		if (v) params.set(key, v);
	}
	return {
		users: await api<Paged<AdminUserRow>>(fetch, `/api/admin/users?${params}`, { token }),
		q: url.searchParams.get('q') ?? '',
		role: url.searchParams.get('role') ?? ''
	};
};
