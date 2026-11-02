<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { categoryName } from '$lib/i18n';
	import BuyButtons from '$lib/components/BuyButtons.svelte';
	import Price from '$lib/components/Price.svelte';
	import Rating from '$lib/components/Rating.svelte';
	import Reviews from '$lib/components/Reviews.svelte';
	import { formatNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const ebook = $derived(data.ebook);
</script>

<svelte:head>
	<title>{ebook.title} · Reanit Academy</title>
	{#if ebook.summary}<meta name="description" content={ebook.summary} />{/if}
</svelte:head>

<div class="grid gap-8 md:grid-cols-[280px_1fr]">
	<div class="aspect-[3/4] overflow-hidden rounded-xl bg-gradient-to-br from-brand-500 to-sky-400 shadow">
		{#if ebook.cover_url}<img src={ebook.cover_url} alt={ebook.title} class="h-full w-full object-cover" />{/if}
	</div>
	<div>
		{#if ebook.category_name_km}
			<a href="/ebooks?category={ebook.category_slug}" class="text-sm text-brand-600">#{categoryName(ebook)}</a>
		{/if}
		<h1 class="mt-1 text-3xl font-bold">{ebook.title}</h1>
		<p class="mt-1 text-slate-600">{m.ebook_author_label()} <strong>{ebook.author_name}</strong></p>
		<div class="mt-2 flex items-center gap-4 text-sm text-slate-600">
			<Rating avg={ebook.rating_avg} count={ebook.rating_count} />
			{#if ebook.page_count}<span>{m.pages_count({ count: ebook.page_count, n: formatNumber(ebook.page_count) })}</span>{/if}
		</div>
		<div class="mt-6"><Price price={ebook.price_cents} sale={ebook.sale_price_cents} size="lg" /></div>
		<div class="mt-4">
			{#if ebook.owned}
				<p class="text-sm text-green-700">✓ {m.ebook_owned()}</p>
				<div class="mt-3 flex flex-wrap gap-2">
					<a href="/ebooks/{ebook.slug}/read" class="rounded-lg bg-brand-600 px-6 py-2.5 font-semibold text-white hover:bg-brand-700">
						{m.ebook_read_now()}
					</a>
					<a
						href="/ebooks/{ebook.slug}/download"
						data-sveltekit-reload
						class="rounded-lg border border-brand-600 px-6 py-2.5 font-semibold text-brand-600 hover:bg-brand-50"
					>
						{m.ebook_download()}
					</a>
				</div>
			{:else}
				<BuyButtons {form} />
			{/if}
		</div>
		{#if ebook.summary}<p class="mt-8 text-lg text-slate-700">{ebook.summary}</p>{/if}
		{#if ebook.description}<p class="mt-4 whitespace-pre-line text-slate-700">{ebook.description}</p>{/if}
		<Reviews reviews={ebook.reviews} myReview={ebook.my_review} owned={ebook.owned} {form} />
	</div>
</div>
