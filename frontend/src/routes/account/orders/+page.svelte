<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import OrderStatusBadge from '$lib/components/OrderStatusBadge.svelte';
	import { formatDateTime, formatPrice } from '$lib/format';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>{m.nav_orders()} · Reanit Academy</title></svelte:head>

<h1 class="text-3xl font-bold">{m.nav_orders()}</h1>

{#if data.orders.length === 0}
	<p class="mt-12 text-center text-slate-500">{m.orders_empty()} <a href="/courses" class="text-brand-600 hover:underline">{m.browse_courses()}</a></p>
{:else}
	<ul class="mt-6 divide-y divide-slate-200 rounded-xl border border-slate-200 bg-white">
		{#each data.orders as o (o.id)}
			<li>
				<a href="/orders/{o.id}" class="flex flex-wrap items-center justify-between gap-4 p-4 hover:bg-slate-50">
					<div class="min-w-0">
						<div class="flex items-center gap-2">
							<span class="font-mono text-sm font-semibold">#{o.id.slice(0, 8).toUpperCase()}</span>
							<OrderStatusBadge status={o.status} />
						</div>
						<p class="mt-1 line-clamp-1 text-sm text-slate-600">{o.items.map((i) => i.title).join(' · ')}</p>
						<p class="text-xs text-slate-400">{formatDateTime(o.created_at)}</p>
					</div>
					<span class="font-semibold">{formatPrice(o.total_cents)}</span>
				</a>
			</li>
		{/each}
	</ul>
{/if}
