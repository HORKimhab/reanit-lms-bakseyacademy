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

/** Only same-site paths, so `?redirectTo=` can't send users to another site. */
export function safeRedirect(target: string | null, fallback = '/'): string {
	return target && target.startsWith('/') && !target.startsWith('//') && !target.startsWith('/\\') ? target : fallback;
}
