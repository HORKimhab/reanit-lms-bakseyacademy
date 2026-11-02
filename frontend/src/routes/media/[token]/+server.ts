import { apiStream } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Telegram-stored media, streamed from the private API. The signed token in the URL is the access
// grant (minted by the lesson / ebook-file endpoints after their checks), so no session is needed.
// `Range` goes through for video seeking and PDF viewers; the body is piped, never buffered.
const PASS_THROUGH = [
	'content-type',
	'content-length',
	'content-range',
	'accept-ranges',
	'content-disposition',
	'cache-control',
	'content-security-policy',
	'x-content-type-options'
];

export const GET: RequestHandler = async ({ params, request, fetch }) => {
	const headers: Record<string, string> = {};
	const range = request.headers.get('range');
	if (range) headers.range = range;
	const res = await apiStream(fetch, `/api/media/${encodeURIComponent(params.token)}`, { method: 'GET', headers });
	const out = new Headers();
	for (const name of PASS_THROUGH) {
		const value = res.headers.get(name);
		if (value) out.set(name, value);
	}
	return new Response(res.body, { status: res.status, headers: out });
};
