import { api } from '$lib/server/api';
import type { Home } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => ({
	home: await api<Home>(fetch, '/api/home')
});
