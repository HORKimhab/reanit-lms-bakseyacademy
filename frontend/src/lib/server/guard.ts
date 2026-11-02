import { redirect } from '@sveltejs/kit';

/** Returns the session token, or sends signed-out users to login and back to `path`. */
export function requireToken(locals: App.Locals, path: string): string {
	if (!locals.token) redirect(303, `/auth?redirectTo=${encodeURIComponent(path)}`);
	return locals.token;
}
