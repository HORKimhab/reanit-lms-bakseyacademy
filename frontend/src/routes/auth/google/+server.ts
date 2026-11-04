import { redirect } from '@sveltejs/kit';
import { apiResult } from '$lib/server/api';
import { startAttempt } from '$lib/server/google';
import { safeRedirect } from '$lib/server/session';
import type { RequestHandler } from './$types';

/** Starts Google sign-in (`?redirectTo=`) or linking (`?mode=link`, signed in) and sends the browser to Google. */
export const GET: RequestHandler = async ({ url, cookies, fetch, locals }) => {
	const mode = url.searchParams.get('mode') === 'link' ? 'link' : 'login';
	if (mode === 'link' && !locals.user) redirect(303, '/auth?redirectTo=/account');
	const back = mode === 'link' ? '/account' : '/auth';
	const redirectTo = safeRedirect(url.searchParams.get('redirectTo'));
	const { attempt, code_challenge } = startAttempt(cookies, mode, redirectTo);
	const res = await apiResult<{ url: string }>(fetch, '/api/auth/google/url', {
		method: 'POST',
		body: { state: attempt.state, nonce: attempt.nonce, code_challenge }
	});
	if (!res.ok) redirect(303, `${back}?error=${encodeURIComponent(res.code)}`);
	redirect(303, res.data.url);
};
