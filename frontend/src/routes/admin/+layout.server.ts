import { error, redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

// Admin area: signed-out users log in first; non-admins get a plain 404 (don't advertise the area).
export const load: LayoutServerLoad = ({ locals, url }) => {
	if (!locals.user) redirect(303, `/auth?redirectTo=${encodeURIComponent(url.pathname + url.search)}`);
	if (locals.user.role !== 'admin') error(404, 'រកមិនឃើញ');
};
