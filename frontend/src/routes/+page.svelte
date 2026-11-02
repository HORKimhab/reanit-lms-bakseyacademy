<script lang="ts">
	import Carousel from '$lib/components/Carousel.svelte';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>Reanit Academy</title></svelte:head>

<Carousel banners={data.home.banners} />

<section class="mt-12">
	<div class="mb-4 flex items-end justify-between">
		<h2 class="text-2xl font-bold">វគ្គសិក្សាថ្មីៗ</h2>
		<a href="/courses" class="text-sm text-brand-600 hover:underline">មើលទាំងអស់ →</a>
	</div>
	<div class="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each data.home.new_courses as c (c.id)}
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
</section>

<section class="mt-12">
	<div class="mb-4 flex items-end justify-between">
		<h2 class="text-2xl font-bold">សៀវភៅថ្មីៗ</h2>
		<a href="/ebooks" class="text-sm text-brand-600 hover:underline">មើលទាំងអស់ →</a>
	</div>
	<div class="grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each data.home.new_ebooks as e (e.id)}
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
</section>
