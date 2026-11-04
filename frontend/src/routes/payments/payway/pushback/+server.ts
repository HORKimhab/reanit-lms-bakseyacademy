import { apiStream } from '$lib/server/api';
import type { RequestHandler } from './$types';

// PayWay's pushback (purchase `return_url` = https://<site>/payments/payway/pushback) relayed to the
// private API. The API trusts nothing in it: it only learns which attempt to re-check with PayWay.
export const POST: RequestHandler = async ({ request, fetch }) => {
	const res = await apiStream(fetch, '/api/payments/payway/pushback', {
		method: 'POST',
		headers: { 'content-type': request.headers.get('content-type') ?? 'application/json' },
		body: request.body
	});
	return new Response(res.body, { status: res.status, headers: { 'content-type': 'application/json' } });
};
