<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { categoryName } from '$lib/i18n';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const recs = $derived(data.recs);
</script>

<svelte:head><title>{m.nav_for_you()} · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{recs.personalized ? m.for_you_title() : m.for_you_popular()}</h1>
<p class="mt-1 text-slate-600">
	{#if recs.personalized}
		{m.for_you_explain()}
	{:else if data.user}
		{m.for_you_buy_first()}
	{:else}
		<a href="/auth?redirectTo=%2Ffor-you" class="text-brand-600 hover:underline">{m.nav_login()}</a> {m.for_you_login_suffix()}
	{/if}
</p>

{#if recs.courses.length}
	<h2 class="mt-8 mb-4 text-xl font-bold">{m.nav_courses()}</h2>
	<div class="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each recs.courses as c (c.id)}
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

{#if recs.ebooks.length}
	<h2 class="mt-10 mb-4 text-xl font-bold">{m.nav_ebooks()}</h2>
	<div class="grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each recs.ebooks as e (e.id)}
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

{#if !recs.courses.length && !recs.ebooks.length}
	<p class="mt-12 text-center text-slate-500">{m.for_you_all_owned()} 🎉</p>
{/if}
