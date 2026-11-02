import { fail, type ActionFailure } from '@sveltejs/kit';
import { apiResult, type ApiOptions } from './api';
import { requireToken } from './guard';

/** Trimmed string field ('' when missing). */
export const text = (d: FormData, key: string) => String(d.get(key) ?? '').trim();

/** Checkbox → boolean. */
export const checked = (d: FormData, key: string) => d.get(key) === 'on';

/**
 * Dollar amount ("15", "15.5", "15.50") → cents. Blank → `null`; malformed → -1 so the API
 * answers `invalid_price` like any other bad price.
 */
export function cents(d: FormData, key: string): number | null {
	const v = text(d, key);
	if (!v) return null;
	return /^\d+(\.\d{1,2})?$/.test(v) ? Math.round(Number(v) * 100) : -1;
}

/** "12:30" or "750" → seconds; blank → 0; malformed → -1 (API answers `invalid_duration`). */
export function seconds(d: FormData, key: string): number {
	const v = text(d, key);
	if (!v) return 0;
	const m = /^(?:(\d+):)?(\d+)$/.exec(v);
	if (!m) return -1;
	return m[1] === undefined ? Number(m[2]) : Number(m[1]) * 60 + Number(m[2]);
}

/** Optional positive integer; blank → null; malformed → -1. */
export function optInt(d: FormData, key: string): number | null {
	const v = text(d, key);
	if (!v) return null;
	return /^\d+$/.test(v) ? Number(v) : -1;
}

type Event = { locals: App.Locals; fetch: typeof fetch; url: URL };

/**
 * Calls an admin endpoint from a form action. Failures become `fail(status, { code, ...extra })`
 * so the page can show the message next to the form that caused it.
 */
export async function adminCall<T, E extends Record<string, string> = Record<never, never>>(
	event: Event,
	path: string,
	opts: Omit<ApiOptions, 'token'>,
	extra?: E
): Promise<{ ok: true; data: T } | { ok: false; failure: ActionFailure<{ code: string } & E> }> {
	const token = requireToken(event.locals, event.url.pathname);
	const res = await apiResult<T>(event.fetch, path, { ...opts, token });
	if (res.ok) return { ok: true, data: res.data };
	return { ok: false, failure: fail(res.status, { code: res.code, ...(extra as E) }) };
}

/**
 * Media for the lesson / ebook pickers: newest 200 of `kind`, or those matching `?mediaq=` (the
 * picker's search box reloads the page with it, so any file in the library can be found).
 */
export function mediaPickerPath(url: URL, kind: 'video' | 'document'): string {
	const params = new URLSearchParams({ kind, per_page: '200' });
	const q = url.searchParams.get('mediaq')?.trim();
	if (q) params.set('q', q);
	return `/api/admin/media?${params}`;
}
