<script lang="ts">
	import ProductCard from '$lib/components/ProductCard.svelte';
	import { khmerNumber } from '$lib/format';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const qs = $derived(new URLSearchParams({ q: data.q }).toString());
</script>

<svelte:head><title>ស្វែងរក "{data.q}" · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">លទ្ធផលស្វែងរក "{data.q}"</h1>

{#if data.courses.total === 0 && data.ebooks.total === 0}
	<p class="mt-12 text-center text-slate-500">រកមិនឃើញវគ្គសិក្សា ឬសៀវភៅទេ។ សូមសាកពាក្យផ្សេង។</p>
{/if}

{#if data.courses.total}
	<div class="mt-8 mb-4 flex items-end justify-between">
		<h2 class="text-xl font-bold">វគ្គសិក្សា ({khmerNumber(data.courses.total)})</h2>
		{#if data.courses.total > data.courses.items.length}
			<a href="/courses?{qs}" class="text-sm text-brand-600 hover:underline">មើលទាំងអស់ →</a>
		{/if}
	</div>
	<div class="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each data.courses.items as c (c.id)}
			<ProductCard
				href="/courses/{c.slug}"
				title={c.title}
				byline={c.instructor_name}
				coverUrl={c.cover_url}
				category={c.category_name_km}
				price={c.price_cents}
				sale={c.sale_price_cents}
				ratingAvg={c.rating_avg}
				ratingCount={c.rating_count}
			/>
		{/each}
	</div>
{/if}

{#if data.ebooks.total}
	<div class="mt-10 mb-4 flex items-end justify-between">
		<h2 class="text-xl font-bold">សៀវភៅ ({khmerNumber(data.ebooks.total)})</h2>
		{#if data.ebooks.total > data.ebooks.items.length}
			<a href="/ebooks?{qs}" class="text-sm text-brand-600 hover:underline">មើលទាំងអស់ →</a>
		{/if}
	</div>
	<div class="grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each data.ebooks.items as e (e.id)}
			<ProductCard
				href="/ebooks/{e.slug}"
				shape="book"
				title={e.title}
				byline={e.author_name}
				coverUrl={e.cover_url}
				category={e.category_name_km}
				price={e.price_cents}
				sale={e.sale_price_cents}
				ratingAvg={e.rating_avg}
				ratingCount={e.rating_count}
			/>
		{/each}
	</div>
{/if}
