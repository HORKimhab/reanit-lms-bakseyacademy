import { redirect } from '@sveltejs/kit';
import { apiResult } from '$lib/server/api';
import { finishAttempt } from '$lib/server/google';
import { safeRedirect, setSession } from '$lib/server/session';
import type { AuthResponse } from '$lib/types';
import type { RequestHandler } from './$types';

/** Google redirects here with `code` + `state`; the API exchanges the code and checks the ID token. */
export const GET: RequestHandler = async ({ url, cookies, fetch, locals }) => {
	const attempt = finishAttempt(cookies, url.searchParams.get('state'));
	const code = url.searchParams.get('code');
	const back = attempt?.mode === 'link' ? '/account' : '/auth';
	if (!attempt || !code) {
		const reason = url.searchParams.get('error') === 'access_denied' ? 'google_cancelled' : 'google_failed';
		redirect(303, `${back}?error=${reason}`);
	}
	const body = { code, code_verifier: attempt.verifier, nonce: attempt.nonce };

	if (attempt.mode === 'link') {
		if (!locals.token) redirect(303, '/auth?redirectTo=/account');
		const res = await apiResult(fetch, '/api/me/google', { method: 'POST', token: locals.token, body });
		redirect(303, res.ok ? '/account?google=linked' : `/account?error=${encodeURIComponent(res.code)}`);
	}
	const res = await apiResult<AuthResponse>(fetch, '/api/auth/google', { method: 'POST', body });
	if (!res.ok) redirect(303, `/auth?error=${encodeURIComponent(res.code)}`);
	setSession(cookies, res.data);
	redirect(303, safeRedirect(attempt.redirectTo));
};
