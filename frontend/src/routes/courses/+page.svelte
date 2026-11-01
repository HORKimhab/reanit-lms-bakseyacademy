<script lang="ts">
	import CatalogSearch from '$lib/components/CatalogSearch.svelte';
	import CategoryChips from '$lib/components/CategoryChips.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>វគ្គសិក្សា · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">វគ្គសិក្សា</h1>
<div class="mt-6 space-y-4">
	<CatalogSearch action="/courses" q={data.q} category={data.category} placeholder="ស្វែងរកវគ្គសិក្សា ឬគ្រូ..." />
	<CategoryChips categories={data.categories} active={data.category} basePath="/courses" q={data.q} />
</div>

{#if data.courses.items.length === 0}
	<p class="mt-12 text-center text-slate-500">រកមិនឃើញវគ្គសិក្សាទេ។</p>
{:else}
	<p class="mt-6 text-sm text-slate-500">{data.courses.total} វគ្គសិក្សា</p>
	<div class="mt-3 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
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
	<Pagination page={data.courses.page} perPage={data.courses.per_page} total={data.courses.total} />
{/if}
