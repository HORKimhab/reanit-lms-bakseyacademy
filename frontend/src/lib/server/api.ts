import { m } from '$lib/paraglide/messages.js';
import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';

const base = () => env.API_URL ?? 'http://127.0.0.1:8080';

export interface ApiOptions {
	method?: 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE';
	/** Sent as JSON. */
	body?: unknown;
	/** Session token from the `session` cookie; sent as `Authorization: Bearer`. */
	token?: string | null;
	/** Sent as multipart instead of JSON `body` (small files from form actions, e.g. screenshots). */
	form?: FormData;
}

export type ApiResult<T> = { ok: true; status: number; data: T } | { ok: false; status: number; code: string };

/**
 * Server-side call to the Rust API that never throws for API errors: returns the `{"error": code}`
 * body instead, for form actions that show validation messages. Network failure → status 503.
 */
export async function apiResult<T>(fetchFn: typeof fetch, path: string, opts: ApiOptions = {}): Promise<ApiResult<T>> {
	const headers: Record<string, string> = {};
	if (opts.body !== undefined && !opts.form) headers['content-type'] = 'application/json';
	if (opts.token) headers.authorization = `Bearer ${opts.token}`;
	let res: Response;
	try {
		res = await fetchFn(`${base()}${path}`, {
			method: opts.method ?? 'GET',
			headers,
			body: opts.form ?? (opts.body === undefined ? undefined : JSON.stringify(opts.body))
		});
	} catch {
		return { ok: false, status: 503, code: 'unavailable' };
	}
	if (res.ok) {
		// 204 / 202 may have no body.
		const text = await res.text();
		const data = (text ? JSON.parse(text) : undefined) as T;
		return { ok: true, status: res.status, data };
	}
	const body = (await res.json().catch(() => null)) as { error?: string } | null;
	return { ok: false, status: res.status, code: body?.error ?? 'unknown' };
}

/** Server-side fetch to the Rust API. Only usable from +page.server.ts / +layout.server.ts. */
export async function api<T>(fetchFn: typeof fetch, path: string, opts: ApiOptions = {}): Promise<T> {
	const res = await apiResult<T>(fetchFn, path, opts);
	if (res.ok) return res.data;
	if (res.status === 503) error(503, m.error_api_unavailable());
	if (res.status === 404) error(404, m.error_not_found());
	error(502, m.error_api_failed());
}

/** Forward only the listing query params the API understands. */
export function listQuery(url: URL): string {
	const params = new URLSearchParams();
	for (const key of ['q', 'category', 'page']) {
		const v = url.searchParams.get(key);
		if (v) params.set(key, v);
	}
	return params.toString();
}

/**
 * Raw streaming call to the Rust API (uploads in, media bytes out): the body and response are
 * piped, never parsed or buffered. Only for `+server.ts` proxy routes; everything else uses `api()`.
 */
export async function apiStream(
	fetchFn: typeof fetch,
	path: string,
	init: { method: 'GET' | 'POST'; token?: string | null; headers?: Record<string, string>; body?: ReadableStream<Uint8Array> | null }
): Promise<Response> {
	const headers: Record<string, string> = { ...init.headers };
	if (init.token) headers.authorization = `Bearer ${init.token}`;
	try {
		return await fetchFn(`${base()}${path}`, {
			method: init.method,
			headers,
			body: init.body ?? undefined,
			// Node's fetch needs this to stream a request body.
			...(init.body ? { duplex: 'half' } : {})
		} as RequestInit);
	} catch {
		return new Response(JSON.stringify({ error: 'unavailable' }), { status: 503, headers: { 'content-type': 'application/json' } });
	}
}
