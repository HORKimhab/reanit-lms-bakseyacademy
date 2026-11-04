import { api } from '$lib/server/api';
import type { BundleCard, Paged } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, url }) => {
	const page = url.searchParams.get('page');
	return { bundles: await api<Paged<BundleCard>>(fetch, `/api/bundles?per_page=24${page ? `&page=${encodeURIComponent(page)}` : ''}`) };
};
