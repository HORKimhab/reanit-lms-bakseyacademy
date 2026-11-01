import { api, listQuery } from '$lib/server/api';
import type { Category, CourseCard, Paged } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, url }) => {
	const [courses, categories] = await Promise.all([
		api<Paged<CourseCard>>(fetch, `/api/courses?${listQuery(url)}`),
		api<Category[]>(fetch, '/api/categories')
	]);
	return {
		courses,
		categories,
		q: url.searchParams.get('q') ?? '',
		category: url.searchParams.get('category') ?? ''
	};
};
