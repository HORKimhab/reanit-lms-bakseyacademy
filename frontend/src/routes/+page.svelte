<script lang="ts">
	import Seo from '$lib/components/Seo.svelte';
	import { getLocale } from '$lib/paraglide/runtime';
	import { site } from '$lib/site';
	import { m } from '$lib/paraglide/messages.js';
	import Carousel from '$lib/components/Carousel.svelte';
	import ProductCard from '$lib/components/ProductCard.svelte';
	import { categoryName } from '$lib/i18n';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<Seo title={site.name} description={site.tagline[getLocale()]} />

<Carousel banners={data.home.banners} />

<section class="mt-12">
	<div class="mb-4 flex items-end justify-between">
		<h2 class="text-2xl font-bold">{m.home_new_courses()}</h2>
		<a href="/courses" class="text-sm text-brand-600 hover:underline">{m.see_all()} →</a>
	</div>
	<div class="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each data.home.new_courses as c (c.id)}
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
</section>

<section class="mt-12">
	<div class="mb-4 flex items-end justify-between">
		<h2 class="text-2xl font-bold">{m.home_new_ebooks()}</h2>
		<a href="/ebooks" class="text-sm text-brand-600 hover:underline">{m.see_all()} →</a>
	</div>
	<div class="grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each data.home.new_ebooks as e (e.id)}
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
</section>
