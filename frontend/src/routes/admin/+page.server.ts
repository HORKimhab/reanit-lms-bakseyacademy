import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { AdminStats } from '$lib/types';
import type { PageServerLoad } from './$types';

const PERIODS = [7, 30, 90, 365];

export const load: PageServerLoad = async ({ locals, fetch, url }) => {
	const token = requireToken(locals, url.pathname);
	const days = Number(url.searchParams.get('days'));
	const period = PERIODS.includes(days) ? days : 30;
	return { stats: await api<AdminStats>(fetch, `/api/admin/stats?days=${period}`, { token }), periods: PERIODS };
};
