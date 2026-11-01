<script lang="ts">
	import CatalogSearch from '$lib/components/CatalogSearch.svelte';
	import CategoryChips from '$lib/components/CategoryChips.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>សៀវភៅ · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">សៀវភៅ</h1>
<div class="mt-6 space-y-4">
	<CatalogSearch action="/ebooks" q={data.q} category={data.category} placeholder="ស្វែងរកសៀវភៅ ឬអ្នកនិពន្ធ..." />
	<CategoryChips categories={data.categories} active={data.category} basePath="/ebooks" q={data.q} />
</div>

{#if data.ebooks.items.length === 0}
	<p class="mt-12 text-center text-slate-500">រកមិនឃើញសៀវភៅទេ។</p>
{:else}
	<p class="mt-6 text-sm text-slate-500">{data.ebooks.total} ក្បាល</p>
	<div class="mt-3 grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
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
	<Pagination page={data.ebooks.page} perPage={data.ebooks.per_page} total={data.ebooks.total} />
{/if}
