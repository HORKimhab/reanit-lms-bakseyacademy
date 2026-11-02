import { api } from '$lib/server/api';
import { addToCartAction } from '$lib/server/cart';
import type { CourseDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, params }) => ({
	course: await api<CourseDetail>(fetch, `/api/courses/${encodeURIComponent(params.slug)}`)
});

export const actions: Actions = {
	addToCart: async (event) => {
		const course = await api<CourseDetail>(event.fetch, `/api/courses/${encodeURIComponent(event.params.slug)}`);
		return addToCartAction(event, { course_id: course.id });
	}
};
