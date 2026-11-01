import { api, listQuery } from '$lib/server/api';
import type { Category, EbookCard, Paged } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, url }) => {
	const [ebooks, categories] = await Promise.all([
		api<Paged<EbookCard>>(fetch, `/api/ebooks?${listQuery(url)}`),
		api<Category[]>(fetch, '/api/categories')
	]);
	return {
		ebooks,
		categories,
		q: url.searchParams.get('q') ?? '',
		category: url.searchParams.get('category') ?? ''
	};
};
