<script lang="ts">
	import { formatNumber } from '$lib/format';
	import { m } from '$lib/paraglide/messages.js';
	import { categoryName } from '$lib/i18n';
	import CatalogSearch from '$lib/components/CatalogSearch.svelte';
	import CategoryChips from '$lib/components/CategoryChips.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>{m.nav_ebooks()} · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{m.nav_ebooks()}</h1>
<div class="mt-6 space-y-4">
	<CatalogSearch action="/ebooks" q={data.q} category={data.category} placeholder={m.ebooks_search_placeholder()} />
	<CategoryChips categories={data.categories} active={data.category} basePath="/ebooks" q={data.q} />
</div>

{#if data.ebooks.items.length === 0}
	<p class="mt-12 text-center text-slate-500">{m.ebooks_none_found()}</p>
{:else}
	<p class="mt-6 text-sm text-slate-500">{m.ebooks_count({ count: data.ebooks.total, n: formatNumber(data.ebooks.total) })}</p>
	<div class="mt-3 grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each data.ebooks.items as e (e.id)}
			<ProductCard
				href="/ebooks/{e.slug}"
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
	<Pagination page={data.ebooks.page} perPage={data.ebooks.per_page} total={data.ebooks.total} />
{/if}
