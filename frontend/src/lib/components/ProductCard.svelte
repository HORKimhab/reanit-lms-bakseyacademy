<script lang="ts">
	import Price from './Price.svelte';
	import Rating from './Rating.svelte';
	import WishlistButton from './WishlistButton.svelte';

	let {
		href,
		title,
		byline,
		coverUrl,
		category,
		price,
		sale,
		ratingAvg,
		ratingCount,
		shape = 'video',
		item = null
	}: {
		href: string;
		title: string;
		byline: string;
		coverUrl: string | null;
		category: string | null;
		price: number;
		sale: number | null;
		ratingAvg: number;
		ratingCount: number;
		shape?: 'video' | 'book';
		/** Shows the wishlist heart. */
		item?: { kind: 'course' | 'ebook'; id: string } | null;
	} = $props();
</script>

<div class="relative flex flex-col">
<a
	{href}
	class="group flex flex-1 flex-col overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
>
	<div
		class="relative overflow-hidden bg-gradient-to-br from-brand-500 to-sky-400 {shape === 'book'
			? 'aspect-[3/4]'
			: 'aspect-video'}"
	>
		{#if coverUrl}
			<img src={coverUrl} alt={title} loading="lazy" class="h-full w-full object-cover transition group-hover:scale-105" />
		{:else}
			<span class="absolute inset-0 grid place-items-center p-4 text-center font-semibold text-white">{title}</span>
		{/if}
	</div>
	<div class="flex flex-1 flex-col gap-1.5 p-4">
		<div class="flex items-center justify-between gap-2">
			<Rating avg={ratingAvg} count={ratingCount} />
			{#if category}<span class="text-xs text-brand-600">#{category}</span>{/if}
		</div>
		<h3 class="line-clamp-2 font-semibold group-hover:text-brand-600">{title}</h3>
		<p class="text-sm text-slate-500">{byline}</p>
		<div class="mt-auto pt-2"><Price {price} {sale} /></div>
	</div>
</a>
{#if item}
	<div class="absolute top-2 right-2"><WishlistButton kind={item.kind} id={item.id} overlay /></div>
{/if}
</div>
