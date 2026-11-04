import type { RequestHandler } from './$types';

/** Keep crawlers on the storefront; point them at the sitemap. */
export const GET: RequestHandler = ({ url }) =>
	new Response(
		[
			'User-agent: *',
			'Disallow: /admin',
			'Disallow: /account',
			'Disallow: /auth',
			'Disallow: /cart',
			'Disallow: /orders',
			'Disallow: /media',
			'Disallow: /search',
			'',
			`Sitemap: ${url.origin}/sitemap.xml`,
			''
		].join('\n'),
		{ headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'public, max-age=86400' } }
	);
