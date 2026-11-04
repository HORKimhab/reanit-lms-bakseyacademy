import { createHash, randomBytes, timingSafeEqual } from 'node:crypto';
import type { Cookies } from '@sveltejs/kit';

/** Short-lived, HttpOnly cookie carrying one Google sign-in attempt between /auth/google and the callback. */
const COOKIE = 'google_oauth';
const PATH = '/auth/google';

export interface GoogleAttempt {
	state: string;
	nonce: string;
	verifier: string;
	mode: 'login' | 'link';
	redirectTo: string;
}

const random = () => randomBytes(32).toString('base64url'); // 43 chars

/** New state / nonce / PKCE verifier, remembered in the cookie; returns the S256 challenge to send. */
export function startAttempt(cookies: Cookies, mode: GoogleAttempt['mode'], redirectTo: string) {
	const attempt: GoogleAttempt = { state: random(), nonce: random(), verifier: random(), mode, redirectTo };
	// SameSite=Lax: the cookie must come back on Google's top-level redirect to the callback.
	cookies.set(COOKIE, JSON.stringify(attempt), { path: PATH, httpOnly: true, sameSite: 'lax', maxAge: 600 });
	const code_challenge = createHash('sha256').update(attempt.verifier).digest('base64url');
	return { attempt, code_challenge };
}

/** The attempt whose `state` matches (constant-time), consumed either way. */
export function finishAttempt(cookies: Cookies, state: string | null): GoogleAttempt | null {
	const raw = cookies.get(COOKIE);
	cookies.delete(COOKIE, { path: PATH });
	if (!raw || !state) return null;
	try {
		const attempt = JSON.parse(raw) as GoogleAttempt;
		const a = Buffer.from(attempt.state);
		const b = Buffer.from(state);
		return a.length === b.length && timingSafeEqual(a, b) ? attempt : null;
	} catch {
		return null;
	}
}
