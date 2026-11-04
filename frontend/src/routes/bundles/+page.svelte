<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import Pagination from '$lib/components/Pagination.svelte';
	import Price from '$lib/components/Price.svelte';
	import Seo from '$lib/components/Seo.svelte';
	import { formatNumber, formatPrice } from '$lib/format';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<Seo title={m.bundles_title()} />

<h1 class="text-3xl font-bold">{m.bundles_title()}</h1>

{#if data.bundles.items.length === 0}
	<p class="mt-12 text-center text-slate-500">{m.bundles_empty()}</p>
{:else}
	<div class="mt-6 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
		{#each data.bundles.items as b (b.id)}
			<a href="/bundles/{b.slug}" class="group flex flex-col overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm transition hover:-translate-y-0.5 hover:shadow-md">
				<div class="relative aspect-video overflow-hidden bg-gradient-to-br from-brand-600 to-fuchsia-400">
					{#if b.cover_url}
						<img src={b.cover_url} alt={b.title} loading="lazy" class="h-full w-full object-cover transition group-hover:scale-105" />
					{:else}
						<span class="absolute inset-0 grid place-items-center p-4 text-center text-lg font-semibold text-white">{b.title}</span>
					{/if}
					<span class="absolute top-2 left-2 rounded-full bg-white/90 px-2 py-0.5 text-xs font-semibold text-brand-700">
						{m.bundle_items_count({ count: b.item_count, n: formatNumber(b.item_count) })}
					</span>
				</div>
				<div class="flex flex-1 flex-col gap-1.5 p-4">
					<h2 class="line-clamp-2 font-semibold group-hover:text-brand-600">{b.title}</h2>
					{#if b.summary}<p class="line-clamp-2 text-sm text-slate-500">{b.summary}</p>{/if}
					<div class="mt-auto flex items-baseline justify-between gap-2 pt-2">
						<Price price={b.price_cents} sale={b.sale_price_cents} />
						{#if b.value_cents > (b.sale_price_cents ?? b.price_cents)}
							<span class="text-xs text-slate-400 line-through">{m.bundle_value({ price: formatPrice(b.value_cents) })}</span>
						{/if}
					</div>
				</div>
			</a>
		{/each}
	</div>
	<div class="mt-8"><Pagination page={data.bundles.page} perPage={data.bundles.per_page} total={data.bundles.total} /></div>
{/if}
