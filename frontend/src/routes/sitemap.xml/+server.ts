import { api } from '$lib/server/api';
import type { Sitemap } from '$lib/types';
import type { RequestHandler } from './$types';

const escape = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

/** Storefront pages plus every published course and ebook. */
export const GET: RequestHandler = async ({ fetch, url }) => {
	const map = await api<Sitemap>(fetch, '/api/sitemap');
	const entry = (path: string, lastmod?: string) =>
		`<url><loc>${escape(url.origin + path)}</loc>${lastmod ? `<lastmod>${lastmod.slice(0, 10)}</lastmod>` : ''}</url>`;
	const urls = [
		entry('/'),
		entry('/courses'),
		entry('/ebooks'),
		...map.courses.map((c) => entry(`/courses/${encodeURIComponent(c.slug)}`, c.updated_at)),
		...map.ebooks.map((e) => entry(`/ebooks/${encodeURIComponent(e.slug)}`, e.updated_at))
	];
	const xml = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.join('\n')}\n</urlset>\n`;
	return new Response(xml, { headers: { 'content-type': 'application/xml; charset=utf-8', 'cache-control': 'public, max-age=3600' } });
};
