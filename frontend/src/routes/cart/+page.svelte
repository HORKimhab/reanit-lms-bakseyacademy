<script lang="ts">
	import { enhance } from '$app/forms';
	import Price from '$lib/components/Price.svelte';
	import { errorMessage } from '$lib/errors';
	import { formatPrice, khmerNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const cart = $derived(data.cart);
	let selected = $state('');
	$effect(() => {
		if (!selected && data.methods.length) selected = data.methods[0].code;
	});
</script>

<svelte:head><title>កន្ត្រក · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">កន្ត្រក</h1>

{#if form?.code}
	<p class="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{/if}

{#if cart.items.length === 0}
	<div class="mt-12 text-center text-slate-500">
		<p>កន្ត្រករបស់អ្នកទទេ។</p>
		<p class="mt-4 space-x-4">
			<a href="/courses" class="text-brand-600 hover:underline">មើលវគ្គសិក្សា</a>
			<a href="/ebooks" class="text-brand-600 hover:underline">មើលសៀវភៅ</a>
		</p>
	</div>
{:else}
	<div class="mt-6 grid gap-8 lg:grid-cols-[1fr_360px]">
		<ul class="divide-y divide-slate-200 rounded-xl border border-slate-200 bg-white">
			{#each cart.items as item (item.id)}
				<li class="flex items-center gap-4 p-4">
					<a
						href="/{item.kind === 'course' ? 'courses' : 'ebooks'}/{item.slug}"
						class="shrink-0 overflow-hidden rounded-lg bg-gradient-to-br from-brand-500 to-sky-400 {item.kind === 'course'
							? 'aspect-video w-28'
							: 'aspect-[3/4] w-16'}"
					>
						{#if item.cover_url}<img src={item.cover_url} alt="" class="h-full w-full object-cover" />{/if}
					</a>
					<div class="min-w-0 flex-1">
						<span class="text-xs text-slate-400">{item.kind === 'course' ? 'វគ្គសិក្សា' : 'សៀវភៅ'}</span>
						<a href="/{item.kind === 'course' ? 'courses' : 'ebooks'}/{item.slug}" class="line-clamp-2 font-semibold hover:text-brand-600">
							{item.title}
						</a>
						<p class="text-sm text-slate-500">{item.byline}</p>
					</div>
					<div class="text-right">
						<Price price={item.price_cents} sale={item.sale_price_cents} />
						<form method="POST" action="?/remove" use:enhance>
							<input type="hidden" name="id" value={item.id} />
							<button class="mt-1 text-sm text-red-600 hover:underline">លុប</button>
						</form>
					</div>
				</li>
			{/each}
		</ul>

		<form method="POST" action="?/checkout" class="space-y-4 self-start rounded-xl border border-slate-200 bg-white p-5 shadow-sm lg:sticky lg:top-24">
			<div class="flex items-baseline justify-between">
				<span class="text-slate-600">សរុប ({khmerNumber(cart.items.length)})</span>
				<span class="text-2xl font-bold text-brand-600">{formatPrice(cart.total_cents)}</span>
			</div>

			{#if cart.total_cents > 0}
				<fieldset class="space-y-2">
					<legend class="mb-2 text-sm font-medium">ជ្រើសរើសវិធីបង់ប្រាក់</legend>
					{#each data.methods as m (m.code)}
						<label
							class="flex cursor-pointer items-center gap-3 rounded-lg border p-3 {selected === m.code
								? 'border-brand-600 bg-brand-50'
								: 'border-slate-200 hover:border-brand-500'}"
						>
							<input type="radio" name="payment_method" value={m.code} bind:group={selected} class="accent-brand-600" />
							<span>
								<span class="block font-semibold">{m.name_km}</span>
								<span class="block text-xs text-slate-500">ស្កេន QR ហើយបញ្ចូលលេខប្រតិបត្តិការ</span>
							</span>
						</label>
					{:else}
						<p class="text-sm text-red-700">មិនទាន់មានវិធីបង់ប្រាក់ទេ។</p>
					{/each}
				</fieldset>
			{/if}

			<button class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700">
				{cart.total_cents > 0 ? 'បន្តទៅការបង់ប្រាក់' : 'ទទួលយកដោយឥតគិតថ្លៃ'}
			</button>
		</form>
	</div>
{/if}
