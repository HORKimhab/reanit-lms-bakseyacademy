import { fail, type RequestEvent } from '@sveltejs/kit';
import { apiResult } from './api';
import { requireToken } from './guard';

type Kind = 'courses' | 'ebooks';

/** Shared `?/review` (save) action for course and ebook detail pages. */
export async function saveReviewAction(event: RequestEvent, kind: Kind, slug: string) {
	const token = requireToken(event.locals, event.url.pathname);
	const data = await event.request.formData();
	const res = await apiResult(event.fetch, `/api/${kind}/${encodeURIComponent(slug)}/review`, {
		method: 'PUT',
		token,
		body: { rating: Number(data.get('rating') ?? 0), comment: String(data.get('comment') ?? '') }
	});
	if (!res.ok) return fail(res.status, { reviewCode: res.code });
	return { reviewSaved: true };
}

/** Shared `?/deleteReview` action. */
export async function deleteReviewAction(event: RequestEvent, kind: Kind, slug: string) {
	const token = requireToken(event.locals, event.url.pathname);
	const res = await apiResult(event.fetch, `/api/${kind}/${encodeURIComponent(slug)}/review`, { method: 'DELETE', token });
	if (!res.ok) return fail(res.status, { reviewCode: res.code });
	return { reviewDeleted: true };
}
