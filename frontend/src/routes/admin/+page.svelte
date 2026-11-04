<script lang="ts">
	import { formatNumber, formatPrice } from '$lib/format';
	import type { AdminTopItem } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const s = $derived(data.stats);
	const max = $derived(Math.max(1, ...s.daily.map((d) => d.revenue_cents)));
	const dayLabel = (iso: string) => formatNumber(`${Number(iso.slice(8, 10))}/${Number(iso.slice(5, 7))}`);
	const periodLabel = (d: number) => (d === 365 ? '១ ឆ្នាំ' : `${formatNumber(d)} ថ្ងៃ`);
</script>

<svelte:head><title>សង្ខេប · Admin</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">សង្ខេប</h1>
	<nav class="flex gap-1 rounded-lg border border-slate-200 bg-white p-1 text-sm">
		{#each data.periods as d (d)}
			<a href="?days={d}" class="rounded-md px-3 py-1 {s.days === d ? 'bg-brand-600 text-white' : 'hover:bg-slate-100'}">{periodLabel(d)}</a>
		{/each}
	</nav>
</div>

<div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
	<a href="/admin/orders" class="rounded-xl border p-4 {s.awaiting_approval ? 'border-amber-300 bg-amber-50' : 'border-slate-200 bg-white'}">
		<p class="text-sm text-slate-600">រង់ចាំអនុម័ត</p>
		<p class="mt-1 text-3xl font-bold">{formatNumber(s.awaiting_approval)}</p>
		<p class="text-xs text-slate-500">មិនទាន់បង់ {formatNumber(s.pending_payment)}</p>
	</a>
	<div class="rounded-xl border border-slate-200 bg-white p-4">
		<p class="text-sm text-slate-600">ចំណូល ({periodLabel(s.days)})</p>
		<p class="mt-1 text-3xl font-bold">{formatPrice(s.revenue_cents)}</p>
		<p class="text-xs text-slate-500">{formatNumber(s.paid_orders)} ការបញ្ជាទិញបានបង់</p>
	</div>
	<div class="rounded-xl border border-slate-200 bg-white p-4">
		<p class="text-sm text-slate-600">ចំណូលសរុប</p>
		<p class="mt-1 text-3xl font-bold">{formatPrice(s.revenue_cents_all_time)}</p>
	</div>
	<div class="rounded-xl border border-slate-200 bg-white p-4">
		<p class="text-sm text-slate-600">អ្នកប្រើប្រាស់</p>
		<p class="mt-1 text-3xl font-bold">{formatNumber(s.users)}</p>
		<p class="text-xs text-slate-500">ថ្មី {formatNumber(s.new_users)} ({periodLabel(s.days)})</p>
	</div>
</div>

<section class="mt-8 rounded-xl border border-slate-200 bg-white p-4">
	<h2 class="font-semibold">ចំណូលប្រចាំថ្ងៃ</h2>
	<div class="mt-4 flex h-48 items-end gap-px" role="img" aria-label="ចំណូលប្រចាំថ្ងៃ">
		{#each s.daily as d (d.day)}
			<div class="group relative flex h-full flex-1 items-end">
				<div class="w-full rounded-t bg-brand-500 group-hover:bg-brand-700" style="height: {(d.revenue_cents / max) * 100}%; min-height: {d.revenue_cents ? 2 : 0}px"></div>
				<div class="pointer-events-none absolute bottom-full left-1/2 z-10 mb-1 hidden -translate-x-1/2 rounded bg-slate-900 px-2 py-1 text-xs whitespace-nowrap text-white group-hover:block">
					{dayLabel(d.day)}: {formatPrice(d.revenue_cents)} · {formatNumber(d.orders)}
				</div>
			</div>
		{/each}
	</div>
	<div class="mt-1 flex justify-between text-xs text-slate-400">
		<span>{dayLabel(s.daily[0].day)}</span>
		<span>{dayLabel(s.daily[s.daily.length - 1].day)}</span>
	</div>
</section>

{#snippet top(title: string, items: AdminTopItem[], base: string)}
	<section class="rounded-xl border border-slate-200 bg-white p-4">
		<h2 class="font-semibold">{title}</h2>
		{#if items.length === 0}
			<p class="mt-3 text-sm text-slate-500">មិនទាន់មានការលក់</p>
		{:else}
			<ol class="mt-3 space-y-2 text-sm">
				{#each items as item, i (item.id)}
					<li class="flex items-center justify-between gap-4">
						<a href="{base}/{item.id}" class="min-w-0 truncate hover:text-brand-600">{formatNumber(i + 1)}. {item.title}</a>
						<span class="shrink-0 text-slate-600">{formatNumber(item.sales)} · {formatPrice(item.revenue_cents)}</span>
					</li>
				{/each}
			</ol>
		{/if}
	</section>
{/snippet}

<div class="mt-8 grid gap-4 lg:grid-cols-2">
	{@render top('វគ្គសិក្សាលក់ដាច់', s.top_courses, '/admin/courses')}
	{@render top('សៀវភៅលក់ដាច់', s.top_ebooks, '/admin/ebooks')}
</div>
