import { m } from '$lib/paraglide/messages.js';
import { error } from '@sveltejs/kit';
import { apiStream } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Admin file upload → Telegram channel. The multipart body is piped to the API unparsed, so a
// 2 GB video never sits in memory here (a form action would buffer it via request.formData()).
export const POST: RequestHandler = async ({ request, locals, fetch }) => {
	if (locals.user?.role !== 'admin') error(404, m.error_not_found());
	const headers: Record<string, string> = { 'content-type': request.headers.get('content-type') ?? '' };
	const length = request.headers.get('content-length');
	if (length) headers['content-length'] = length;
	const res = await apiStream(fetch, '/api/admin/media', { method: 'POST', token: locals.token, headers, body: request.body });
	return new Response(res.body, { status: res.status, headers: { 'content-type': res.headers.get('content-type') ?? 'application/json' } });
};
