import { m } from '$lib/paraglide/messages.js';
import { error, redirect } from '@sveltejs/kit';
import { apiResult } from './api';
import { requireToken } from './guard';
import type { EbookFile } from '$lib/types';

/**
 * Presigned URL for an owned ebook's file. Non-owners go back to the ebook page; a missing file
 * or unconfigured storage returns `null` so the reader can explain instead of erroring.
 */
export async function ebookFileUrl(
	fetchFn: typeof fetch,
	locals: App.Locals,
	slug: string,
	disposition: 'inline' | 'attachment',
	path: string
): Promise<string | null> {
	const token = requireToken(locals, path);
	const res = await apiResult<EbookFile>(fetchFn, `/api/ebooks/${encodeURIComponent(slug)}/file?disposition=${disposition}`, {
		token
	});
	if (res.ok) return res.data.url;
	if (res.status === 401) redirect(303, `/auth?redirectTo=${encodeURIComponent(path)}`);
	if (res.status === 403) redirect(303, `/ebooks/${slug}`);
	if (res.status === 404 && res.code === 'not_found') return null;
	if (res.status === 503 && res.code === 'storage_unavailable') return null;
	error(502, m.error_api_failed());
}
