import { api } from '$lib/server/api';
import type { Certificate } from '$lib/types';
import type { PageServerLoad } from './$types';

// Public: the link verifies the certificate for anyone (e.g. an employer).
export const load: PageServerLoad = async ({ fetch, params }) => ({
	certificate: await api<Certificate>(fetch, `/api/certificates/${encodeURIComponent(params.id)}`)
});
