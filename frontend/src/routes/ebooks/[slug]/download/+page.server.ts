import { redirect } from '@sveltejs/kit';
import { ebookFileUrl } from '$lib/server/ebook';
import type { PageServerLoad } from './$types';

// Fresh presigned "attachment" link on every click; falls back to the reader's explanation.
export const load: PageServerLoad = async ({ fetch, params, locals, url }) => {
	const fileUrl = await ebookFileUrl(fetch, locals, params.slug, 'attachment', url.pathname);
	redirect(303, fileUrl ?? `/ebooks/${params.slug}/read`);
};
