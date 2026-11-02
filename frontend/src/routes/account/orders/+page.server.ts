import { api } from '$lib/server/api';
import { requireToken } from '$lib/server/guard';
import type { OrderWithItems } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, fetch }) => {
	const token = requireToken(locals, '/account/orders');
	return { orders: await api<OrderWithItems[]>(fetch, '/api/me/orders', { token }) };
};
