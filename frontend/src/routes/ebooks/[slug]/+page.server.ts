import { api } from '$lib/server/api';
import { addToCartAction } from '$lib/server/cart';
import type { EbookDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params }) => ({
	ebook: await api<EbookDetail>(fetch, `/api/ebooks/${encodeURIComponent(params.slug)}`)
});

export const actions: Actions = {
	addToCart: async (event) => {
		const ebook = await api<EbookDetail>(event.fetch, `/api/ebooks/${encodeURIComponent(event.params.slug)}`);
		return addToCartAction(event, { ebook_id: ebook.id });
	}
};
