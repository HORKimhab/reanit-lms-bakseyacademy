import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';

const base = () => env.API_URL ?? 'http://127.0.0.1:8080';

/** Server-side fetch to the Rust API. Only usable from +page.server.ts / +layout.server.ts. */
export async function api<T>(fetchFn: typeof fetch, path: string): Promise<T> {
	let res: Response;
	try {
		res = await fetchFn(`${base()}${path}`);
	} catch {
		error(503, 'មិនអាចភ្ជាប់ទៅម៉ាស៊ីនមេបានទេ');
	}
	if (res.status === 404) error(404, 'រកមិនឃើញ');
	if (!res.ok) error(502, 'មានបញ្ហាពីម៉ាស៊ីនមេ');
	return res.json() as Promise<T>;
}

/** Forward only the listing query params the API understands. */
export function listQuery(url: URL): string {
	const params = new URLSearchParams();
	for (const key of ['q', 'category', 'page']) {
		const v = url.searchParams.get(key);
		if (v) params.set(key, v);
	}
	return params.toString();
}
