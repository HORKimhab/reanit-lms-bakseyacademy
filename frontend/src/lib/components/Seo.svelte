<script lang="ts">
	// Title, description and Open Graph / Twitter tags, so links shared on Facebook / Telegram get a
	// preview card. Relative image paths are made absolute (crawlers need full URLs).
	import { page } from '$app/state';
	import { getLocale } from '$lib/paraglide/runtime';
	import { site } from '$lib/site';

	let {
		title,
		description = null,
		image = null,
		type = 'website'
	}: { title: string; description?: string | null; image?: string | null; type?: 'website' | 'article' | 'book' } = $props();

	const fullTitle = $derived(title === site.name ? title : `${title} · ${site.name}`);
	const url = $derived(page.url.origin + page.url.pathname);
	const imageUrl = $derived(image ? new URL(image, page.url.origin).href : null);
</script>

<svelte:head>
	<title>{fullTitle}</title>
	<link rel="canonical" href={url} />
	{#if description}<meta name="description" content={description} />{/if}
	<meta property="og:site_name" content={site.name} />
	<meta property="og:type" content={type} />
	<meta property="og:title" content={title} />
	<meta property="og:url" content={url} />
	<meta property="og:locale" content={getLocale() === 'en' ? 'en_US' : 'km_KH'} />
	{#if description}<meta property="og:description" content={description} />{/if}
	{#if imageUrl}<meta property="og:image" content={imageUrl} />{/if}
	<meta name="twitter:card" content={imageUrl ? 'summary_large_image' : 'summary'} />
</svelte:head>
