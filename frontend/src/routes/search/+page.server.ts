import { redirect } from '@sveltejs/kit';
import { api } from '$lib/server/api';
import type { CourseCard, EbookCard, Paged } from '$lib/types';
import type { PageServerLoad } from './$types';

// One search box for the whole catalog: top courses and ebooks for `q`, each linking to its full list.
export const load: PageServerLoad = async ({ fetch, url }) => {
	const q = url.searchParams.get('q')?.trim() ?? '';
	if (!q) redirect(303, '/courses');
	const params = new URLSearchParams({ q, per_page: '8' });
	const [courses, ebooks] = await Promise.all([
		api<Paged<CourseCard>>(fetch, `/api/courses?${params}`),
		api<Paged<EbookCard>>(fetch, `/api/ebooks?${params}`)
	]);
	return { q, courses, ebooks };
};
