import { api } from '$lib/server/api';
import type { CourseDetail } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params }) => ({
	course: await api<CourseDetail>(fetch, `/api/courses/${encodeURIComponent(params.slug)}`)
});
