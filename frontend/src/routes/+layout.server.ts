import { apiResult } from '$lib/server/api';
import type { Cart } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, fetch, depends }) => {
	depends('app:cart');
	let cartCount = 0;
	if (locals.token) {
		const res = await apiResult<Cart>(fetch, '/api/cart', { token: locals.token });
		if (res.ok) cartCount = res.data.items.length;
	}
	return { user: locals.user, cartCount };
};
