import type { Handle } from '@sveltejs/kit';
import { apiResult } from '$lib/server/api';
import { SESSION_COOKIE, clearSession } from '$lib/server/session';
import type { User } from '$lib/types';

export const handle: Handle = async ({ event, resolve }) => {
	const token = event.cookies.get(SESSION_COOKIE) ?? null;
	event.locals.token = null;
	event.locals.user = null;
	if (token) {
		const res = await apiResult<User>(event.fetch, '/api/me', { token });
		if (res.ok) {
			event.locals.token = token;
			event.locals.user = res.data;
		} else if (res.status === 401) {
			clearSession(event.cookies);
		}
		// Other failures (API down): render signed-out but keep the cookie for the next request.
	}
	return resolve(event);
};
