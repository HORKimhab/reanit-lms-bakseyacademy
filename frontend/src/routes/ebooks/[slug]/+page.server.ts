import { api } from '$lib/server/api';
import type { EbookDetail } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params }) => ({
	ebook: await api<EbookDetail>(fetch, `/api/ebooks/${encodeURIComponent(params.slug)}`)
});
