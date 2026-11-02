import { api } from '$lib/server/api';
import { ebookFileUrl } from '$lib/server/ebook';
import type { EbookDetail } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params, locals, url }) => {
	const fileUrl = await ebookFileUrl(fetch, locals, params.slug, 'inline', url.pathname);
	const ebook = await api<EbookDetail>(fetch, `/api/ebooks/${encodeURIComponent(params.slug)}`, { token: locals.token });
	return { ebook, fileUrl };
};
