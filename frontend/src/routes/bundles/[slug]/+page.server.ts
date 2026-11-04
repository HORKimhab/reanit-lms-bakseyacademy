import { api } from '$lib/server/api';
import { addToCartAction } from '$lib/server/cart';
import type { BundleDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params, locals }) => ({
	bundle: await api<BundleDetail>(fetch, `/api/bundles/${encodeURIComponent(params.slug)}`, { token: locals.token })
});

export const actions: Actions = {
	addToCart: async (event) => {
		const bundle = await api<BundleDetail>(event.fetch, `/api/bundles/${encodeURIComponent(event.params.slug)}`);
		return addToCartAction(event, { bundle_id: bundle.id });
	}
};
