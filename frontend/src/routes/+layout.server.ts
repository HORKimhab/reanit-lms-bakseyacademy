import { apiResult } from '$lib/server/api';
import type { Cart, Notifications, WishlistIds } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, fetch, depends }) => {
	depends('app:cart');
	let cartCount = 0;
	let notifications: Notifications = { items: [], unread_count: 0 };
	let wishlist: WishlistIds = { course_ids: [], ebook_ids: [] };
	if (locals.token) {
		const [cart, notes, saved] = await Promise.all([
			apiResult<Cart>(fetch, '/api/cart', { token: locals.token }),
			apiResult<Notifications>(fetch, '/api/me/notifications', { token: locals.token }),
			apiResult<WishlistIds>(fetch, '/api/me/wishlist/ids', { token: locals.token })
		]);
		if (cart.ok) cartCount = cart.data.items.length;
		if (notes.ok) notifications = notes.data;
		if (saved.ok) wishlist = saved.data;
	}
	return { user: locals.user, cartCount, notifications, wishlist };
};
