import type { Cookies } from '@sveltejs/kit';
import type { AuthResponse } from '$lib/types';

/** HttpOnly cookie holding the API session token. The browser never sees the token from JS. */
export const SESSION_COOKIE = 'session';

export function setSession(cookies: Cookies, auth: AuthResponse) {
	cookies.set(SESSION_COOKIE, auth.token, {
		path: '/',
		httpOnly: true,
		sameSite: 'lax',
		maxAge: auth.max_age
	});
}

export function clearSession(cookies: Cookies) {
	cookies.delete(SESSION_COOKIE, { path: '/' });
}

/**
 * Only same-site paths, so `?redirectTo=` can't send users to another site. Browsers drop tabs and
 * newlines inside URLs (`/\t/evil.com` → `//evil.com`), so control characters are refused outright,
 * and the result must resolve to this origin.
 */
export function safeRedirect(target: string | null, fallback = '/'): string {
	if (!target || !target.startsWith('/') || /[\u0000-\u001f\u007f\\]/.test(target)) return fallback;
	const base = 'http://same.invalid';
	try {
		const url = new URL(target, base);
		return url.origin === base ? url.pathname + url.search + url.hash : fallback;
	} catch {
		return fallback;
	}
}
