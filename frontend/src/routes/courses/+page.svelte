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

<svelte:head><title>{m.nav_courses()} · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{m.nav_courses()}</h1>
<div class="mt-6 space-y-4">
	<CatalogSearch action="/courses" q={data.q} category={data.category} placeholder={m.courses_search_placeholder()} />
	<CategoryChips categories={data.categories} active={data.category} basePath="/courses" q={data.q} />
</div>

{#if data.courses.items.length === 0}
	<p class="mt-12 text-center text-slate-500">{m.courses_none_found()}</p>
{:else}
	<p class="mt-6 text-sm text-slate-500">{m.courses_count({ count: data.courses.total, n: formatNumber(data.courses.total) })}</p>
	<div class="mt-3 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each data.courses.items as c (c.id)}
			<ProductCard
				href="/courses/{c.slug}"
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
	<Pagination page={data.courses.page} perPage={data.courses.per_page} total={data.courses.total} />
{/if}
