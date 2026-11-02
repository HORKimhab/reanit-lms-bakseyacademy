<script lang="ts">
	import ProductCard from '$lib/components/ProductCard.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const recs = $derived(data.recs);
</script>

<svelte:head><title>សម្រាប់អ្នក · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{recs.personalized ? 'ណែនាំសម្រាប់អ្នក' : 'ពេញនិយម'}</h1>
<p class="mt-1 text-slate-600">
	{#if recs.personalized}
		ផ្អែកលើវគ្គសិក្សា និងសៀវភៅដែលអ្នកមាន និងអ្វីដែលអ្នកសិក្សាផ្សេងទៀតបានជ្រើសរើស។
	{:else if data.user}
		ទិញវគ្គសិក្សា ឬសៀវភៅដំបូងរបស់អ្នក ដើម្បីទទួលបានការណែនាំផ្ទាល់ខ្លួន។
	{:else}
		<a href="/auth?redirectTo=%2Ffor-you" class="text-brand-600 hover:underline">ចូលគណនី</a> ដើម្បីទទួលបានការណែនាំផ្ទាល់ខ្លួន។
	{/if}
</p>

{#if recs.courses.length}
	<h2 class="mt-8 mb-4 text-xl font-bold">វគ្គសិក្សា</h2>
	<div class="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
		{#each recs.courses as c (c.id)}
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
{/if}

{#if recs.ebooks.length}
	<h2 class="mt-10 mb-4 text-xl font-bold">សៀវភៅ</h2>
	<div class="grid grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-5">
		{#each recs.ebooks as e (e.id)}
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
{/if}

{#if !recs.courses.length && !recs.ebooks.length}
	<p class="mt-12 text-center text-slate-500">អ្នកមានគ្រប់យ៉ាងហើយ! 🎉</p>
{/if}
