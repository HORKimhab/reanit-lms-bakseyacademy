import type { Handle } from '@sveltejs/kit';
import { sequence } from '@sveltejs/kit/hooks';
import { paraglideMiddleware } from '$lib/paraglide/server';
import { apiResult } from '$lib/server/api';
import { SESSION_COOKIE, clearSession } from '$lib/server/session';
import type { User } from '$lib/types';

/** Resolves the locale (cookie → Khmer) for this request and sets `<html lang>`. */
const i18n: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;
		return resolve(event, {
			transformPageChunk: ({ html }) => html.replace('%paraglide.lang%', locale)
		});
	});

const session: Handle = async ({ event, resolve }) => {
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

export const handle = sequence(i18n, session);
