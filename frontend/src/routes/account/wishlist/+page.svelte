<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import { categoryName } from '$lib/i18n';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const { courses, ebooks } = $derived(data.list);
</script>

<svelte:head><title>{m.nav_wishlist()} · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{m.nav_wishlist()}</h1>

{#if courses.length === 0 && ebooks.length === 0}
	<p class="mt-12 text-center text-slate-500">
		{m.wishlist_empty()} <a href="/courses" class="text-brand-600 hover:underline">{m.browse_courses()}</a>
	</p>
{/if}

{#if courses.length}
	<h2 class="mt-8 text-xl font-bold">{m.nav_courses()}</h2>
	<div class="mt-4 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each courses as c (c.id)}
			<ProductCard
				href="/courses/{c.slug}"
				item={{ kind: 'course', id: c.id }}
				title={c.title}
				byline={c.instructor_name}
				coverUrl={c.cover_url}
				category={categoryName(c)}
				price={c.price_cents}
				sale={c.sale_price_cents}
				ratingAvg={c.rating_avg}
				ratingCount={c.rating_count}
			/>
		{/each}
	</div>
{/if}

{#if ebooks.length}
	<h2 class="mt-10 text-xl font-bold">{m.nav_ebooks()}</h2>
	<div class="mt-4 grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each ebooks as e (e.id)}
			<ProductCard
				href="/ebooks/{e.slug}"
				item={{ kind: 'ebook', id: e.id }}
				shape="book"
				title={e.title}
				byline={e.author_name}
				coverUrl={e.cover_url}
				category={categoryName(e)}
				price={e.price_cents}
				sale={e.sale_price_cents}
				ratingAvg={e.rating_avg}
				ratingCount={e.rating_count}
			/>
		{/each}
	</div>
{/if}
