<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import BuyButtons from '$lib/components/BuyButtons.svelte';
	import Price from '$lib/components/Price.svelte';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import Seo from '$lib/components/Seo.svelte';
	import { formatNumber, formatPrice } from '$lib/format';
	import { categoryName } from '$lib/i18n';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const b = $derived(data.bundle);
	const price = $derived(b.sale_price_cents ?? b.price_cents);
</script>

<Seo title={b.title} description={b.summary} image={b.cover_url} />

<div class="grid gap-8 lg:grid-cols-[1fr_340px]">
	<div>
		<p class="text-sm font-semibold text-brand-600">{m.kind_bundle()} · {m.bundle_items_count({ count: b.item_count, n: formatNumber(b.item_count) })}</p>
		<h1 class="mt-1 text-3xl font-bold">{b.title}</h1>
		{#if b.summary}<p class="mt-2 text-lg text-slate-600">{b.summary}</p>{/if}
		{#if b.description}<p class="mt-6 whitespace-pre-line text-slate-700">{b.description}</p>{/if}

		<h2 class="mt-10 text-xl font-bold">{m.bundle_includes()}</h2>
		<div class="mt-4 grid gap-5 sm:grid-cols-2 xl:grid-cols-3">
			{#each b.courses as c (c.id)}
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
			{#each b.ebooks as e (e.id)}
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
	</div>

	<aside class="lg:sticky lg:top-24 lg:self-start">
		<div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
			<div class="aspect-video bg-gradient-to-br from-brand-600 to-fuchsia-400">
				{#if b.cover_url}<img src={b.cover_url} alt={b.title} class="h-full w-full object-cover" />{/if}
			</div>
			<div class="space-y-4 p-5">
				<Price price={b.price_cents} sale={b.sale_price_cents} size="lg" />
				{#if b.value_cents > price}
					<p class="text-sm text-slate-500"><span class="line-through">{m.bundle_value({ price: formatPrice(b.value_cents) })}</span></p>
				{/if}
				{#if b.owned}
					<p class="rounded-lg bg-green-50 px-3 py-2 text-sm text-green-700">✓ <a href="/account/library" class="underline">{m.bundle_owned()}</a></p>
				{:else}
					<BuyButtons {form} full />
				{/if}
			</div>
		</div>
	</aside>
</div>
