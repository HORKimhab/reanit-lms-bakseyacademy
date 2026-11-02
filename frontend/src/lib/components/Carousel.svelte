<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import type { Banner } from '$lib/types';

	let { banners, interval = 6000 }: { banners: Banner[]; interval?: number } = $props();

	let index = $state(0);
	let paused = $state(false);
	const count = $derived(banners.length);

	function go(i: number) {
		index = (i + count) % count;
	}

	$effect(() => {
		if (count < 2 || paused) return;
		const id = setInterval(() => go(index + 1), interval);
		return () => clearInterval(id);
	});
</script>

{#if count}
	<section
		class="relative overflow-hidden rounded-2xl"
		aria-label={m.carousel_label()}
		aria-roledescription="carousel"
		onmouseenter={() => (paused = true)}
		onmouseleave={() => (paused = false)}
		onfocusin={() => (paused = true)}
		onfocusout={() => (paused = false)}
	>
		<div class="flex transition-transform duration-500" style="transform: translateX(-{index * 100}%)">
			{#each banners as b, i (b.id)}
				<div
					class="relative aspect-[16/9] w-full shrink-0 bg-gradient-to-br from-brand-600 to-sky-500 sm:aspect-[3/1]"
					aria-roledescription="slide"
					aria-hidden={i !== index}
				>
					{#if b.image_url}
						<img src={b.image_url} alt="" class="absolute inset-0 h-full w-full object-cover" loading={i ? 'lazy' : 'eager'} />
						<div class="absolute inset-0 bg-gradient-to-r from-black/60 to-transparent"></div>
					{/if}
					<div class="relative flex h-full flex-col justify-center px-6 text-white sm:px-12">
						<h2 class="max-w-xl text-2xl font-bold sm:text-4xl">{b.title}</h2>
						{#if b.subtitle}<p class="mt-3 max-w-xl text-white/85">{b.subtitle}</p>{/if}
						{#if b.link_url}
							<a
								href={b.link_url}
								tabindex={i === index ? 0 : -1}
								class="mt-6 inline-block w-fit rounded-lg bg-white px-5 py-2.5 font-semibold text-brand-700 hover:bg-brand-50"
							>
								{m.carousel_more()}
							</a>
						{/if}
					</div>
				</div>
			{/each}
		</div>

		{#if count > 1}
			<button
				class="absolute top-1/2 left-3 hidden -translate-y-1/2 rounded-full bg-white/80 px-3 py-1.5 text-slate-700 hover:bg-white sm:block"
				aria-label={m.carousel_prev()}
				onclick={() => go(index - 1)}>‹</button
			>
			<button
				class="absolute top-1/2 right-3 hidden -translate-y-1/2 rounded-full bg-white/80 px-3 py-1.5 text-slate-700 hover:bg-white sm:block"
				aria-label={m.carousel_next()}
				onclick={() => go(index + 1)}>›</button
			>
			<div class="absolute bottom-3 left-1/2 flex -translate-x-1/2 gap-2">
				{#each banners as b, i (b.id)}
					<button
						class="h-2 rounded-full transition-all {i === index ? 'w-6 bg-white' : 'w-2 bg-white/50'}"
						aria-label={m.carousel_slide({ n: i + 1 })}
						aria-current={i === index}
						onclick={() => go(i)}
					></button>
				{/each}
			</div>
		{/if}
	</section>
{/if}
