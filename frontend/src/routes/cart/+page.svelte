<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { localized } from '$lib/i18n';
	import { enhance } from '$app/forms';
	import Price from '$lib/components/Price.svelte';
	import { errorMessage } from '$lib/errors';
	import { formatPrice, formatNumber } from '$lib/format';
	import { kindLabel, kindPath } from '$lib/i18n';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const cart = $derived(data.cart);
	let selected = $state('');
	$effect(() => {
		if (!selected && data.methods.length) selected = data.methods[0].code;
	});
</script>

<svelte:head><title>{m.nav_cart()} · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{m.nav_cart()}</h1>

{#if form?.code}
	<p class="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{/if}

{#if cart.items.length === 0}
	<div class="mt-12 text-center text-slate-500">
		<p>{m.cart_empty_message()}</p>
		<p class="mt-4 space-x-4">
			<a href="/courses" class="text-brand-600 hover:underline">{m.browse_courses()}</a>
			<a href="/ebooks" class="text-brand-600 hover:underline">{m.browse_ebooks()}</a>
		</p>
	</div>
{:else}
	<div class="mt-6 grid gap-8 lg:grid-cols-[1fr_360px]">
		<ul class="divide-y divide-slate-200 rounded-xl border border-slate-200 bg-white">
			{#each cart.items as item (item.id)}
				<li class="flex items-center gap-4 p-4">
					<a
						href="/{kindPath(item.kind)}/{item.slug}"
						class="shrink-0 overflow-hidden rounded-lg bg-gradient-to-br from-brand-500 to-sky-400 {item.kind !== 'ebook'
							? 'aspect-video w-28'
							: 'aspect-[3/4] w-16'}"
					>
						{#if item.cover_url}<img src={item.cover_url} alt="" class="h-full w-full object-cover" />{/if}
					</a>
					<div class="min-w-0 flex-1">
						<span class="text-xs text-slate-400">{kindLabel(item.kind)}</span>
						<a href="/{kindPath(item.kind)}/{item.slug}" class="line-clamp-2 font-semibold hover:text-brand-600">
							{item.title}
						</a>
						<p class="line-clamp-2 text-sm text-slate-500">{item.byline}</p>
					</div>
					<div class="text-right">
						<Price price={item.price_cents} sale={item.sale_price_cents} />
						<form method="POST" action="?/remove" use:enhance>
							<input type="hidden" name="id" value={item.id} />
							<button class="mt-1 text-sm text-red-600 hover:underline">{m.remove()}</button>
						</form>
					</div>
				</li>
			{/each}
		</ul>

		<div class="space-y-4 self-start rounded-xl border border-slate-200 bg-white p-5 shadow-sm lg:sticky lg:top-24">
		<form method="GET" class="space-y-1">
			<label for="coupon" class="text-sm font-medium">{m.coupon_label()}</label>
			<div class="flex gap-2">
				<input
					id="coupon"
					name="coupon"
					value={cart.coupon?.code ?? data.coupon}
					autocomplete="off"
					class="min-w-0 flex-1 rounded-lg border border-slate-200 px-3 py-1.5 font-mono text-sm uppercase outline-none focus:border-brand-500"
				/>
				<button class="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium hover:bg-slate-100">{m.coupon_apply()}</button>
			</div>
			{#if data.couponError}
				<p class="text-sm text-red-700" role="alert">{errorMessage(data.couponError)}</p>
			{:else if cart.coupon}
				<p class="text-sm text-green-700" role="status">
					✓ {m.coupon_applied({ code: cart.coupon.code })} · <a href="/cart" class="underline">{m.coupon_remove()}</a>
				</p>
			{/if}
		</form>

		<form method="POST" action="?/checkout" class="space-y-4">
			{#if cart.coupon}
				<input type="hidden" name="coupon_code" value={cart.coupon.code} />
				<div class="space-y-1 text-sm text-slate-600">
					<p class="flex justify-between"><span>{m.subtotal()}</span><span>{formatPrice(cart.subtotal_cents)}</span></p>
					<p class="flex justify-between text-green-700"><span>{m.discount()} ({cart.coupon.code})</span><span>−{formatPrice(cart.discount_cents)}</span></p>
				</div>
			{/if}
			<div class="flex items-baseline justify-between">
				<span class="text-slate-600">{m.total()} ({formatNumber(cart.items.length)})</span>
				<span class="text-2xl font-bold text-brand-600">{formatPrice(cart.total_cents)}</span>
			</div>

			{#if cart.total_cents > 0}
				<fieldset class="space-y-2">
					<legend class="mb-2 text-sm font-medium">{m.cart_choose_method()}</legend>
					{#each data.methods as pm (pm.code)}
						<label
							class="flex cursor-pointer items-center gap-3 rounded-lg border p-3 {selected === pm.code
								? 'border-brand-600 bg-brand-50'
								: 'border-slate-200 hover:border-brand-500'}"
						>
							<input type="radio" name="payment_method" value={pm.code} bind:group={selected} class="accent-brand-600" />
							<span>
								<span class="block font-semibold">{localized(pm.name_km, pm.name_en)}</span>
								<span class="block text-xs text-slate-500">{m.cart_method_hint()}</span>
							</span>
						</label>
					{:else}
						<p class="text-sm text-red-700">{m.cart_no_methods()}</p>
					{/each}
				</fieldset>
			{/if}

			<button class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700">
				{cart.total_cents > 0 ? m.cart_checkout() : m.cart_claim_free()}
			</button>
		</form>
		</div>
	</div>
{/if}
