import { fail, redirect, type RequestEvent } from '@sveltejs/kit';
import { apiResult } from './api';

/**
 * Shared `?/addToCart` action for course and ebook detail pages.
 * Signed-out users go to login and come back. A `next=cart` field ("buy now") continues to /cart.
 */
export async function addToCartAction(event: RequestEvent, item: { course_id: string } | { ebook_id: string }) {
	const { locals, fetch, url, request } = event;
	if (!locals.token) redirect(303, `/auth?redirectTo=${encodeURIComponent(url.pathname)}`);
	const data = await request.formData();
	const res = await apiResult(fetch, '/api/cart', { method: 'POST', token: locals.token, body: item });
	if (!res.ok) return fail(res.status, { code: res.code });
	if (data.get('next') === 'cart') redirect(303, '/cart');
	return { added: true };
}
