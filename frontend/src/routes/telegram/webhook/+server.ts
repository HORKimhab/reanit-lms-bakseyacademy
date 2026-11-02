import { apiStream } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Telegram's webhook (setWebhook url = https://<site>/telegram/webhook) relayed to the private API,
// which checks the secret token and imports storage-channel posts into the media library.
export const POST: RequestHandler = async ({ request, fetch }) => {
	const res = await apiStream(fetch, '/api/telegram/webhook', {
		method: 'POST',
		headers: {
			'content-type': request.headers.get('content-type') ?? 'application/json',
			'x-telegram-bot-api-secret-token': request.headers.get('x-telegram-bot-api-secret-token') ?? ''
		},
		body: request.body
	});
	return new Response(null, { status: res.status });
};
