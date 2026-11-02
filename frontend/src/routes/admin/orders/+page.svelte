<script lang="ts">
	import { enhance } from '$app/forms';
	import OrderStatusBadge from '$lib/components/OrderStatusBadge.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime, formatPrice, formatNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const tabs = [
		{ status: 'awaiting_approval', label: 'រង់ចាំអនុម័ត' },
		{ status: 'pending_payment', label: 'មិនទាន់បង់' },
		{ status: 'rejected', label: 'បានបដិសេធ' },
		{ status: 'paid', label: 'បានបង់' },
		{ status: 'cancelled', label: 'បានបោះបង់' },
		{ status: 'all', label: 'ទាំងអស់' }
	];
	const tabHref = (status: string) => {
		const p = new URLSearchParams();
		if (status !== 'awaiting_approval') p.set('status', status);
		if (data.q) p.set('q', data.q);
		const s = p.toString();
		return s ? `?${s}` : '/admin/orders';
	};
	const canApprove = (s: string) => s === 'awaiting_approval' || s === 'pending_payment' || s === 'rejected';
</script>

<svelte:head><title>ការបញ្ជាទិញ · Admin</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">ការបញ្ជាទិញ</h1>
	<form method="GET" class="flex gap-2">
		{#if data.status !== 'awaiting_approval'}<input type="hidden" name="status" value={data.status} />{/if}
		<input
			name="q"
			type="search"
			value={data.q}
			placeholder="ឈ្មោះ លេខទូរស័ព្ទ ឬលេខប្រតិបត្តិការ"
			class="w-72 rounded-lg border border-slate-200 px-3 py-1.5 text-sm outline-none focus:border-brand-500"
		/>
		<button class="rounded-lg border border-slate-200 px-3 text-sm hover:border-brand-500">ស្វែងរក</button>
	</form>
</div>

<nav class="mt-4 flex flex-wrap gap-2">
	{#each tabs as t (t.status)}
		<a
			href={tabHref(t.status)}
			class="rounded-full border px-3 py-1 text-sm {data.status === t.status
				? 'border-brand-600 bg-brand-600 text-white'
				: 'border-slate-200 bg-white hover:border-brand-500'}">{t.label}</a
		>
	{/each}
</nav>

{#if form && 'done' in form && form.done}
	<p class="mt-4 rounded-lg bg-green-50 px-3 py-2 text-sm text-green-700" role="status">
		#{form.id.slice(0, 8).toUpperCase()} — {form.done === 'approved' ? 'បានអនុម័ត និងផ្តល់សិទ្ធិចូលប្រើ' : 'បានបដិសេធ'}
	</p>
{/if}

<p class="mt-4 text-sm text-slate-500">{formatNumber(data.orders.total)} ការបញ្ជាទិញ</p>

{#if data.orders.items.length === 0}
	<p class="mt-10 text-center text-slate-500">មិនមានការបញ្ជាទិញទេ។</p>
{:else}
	<ul class="mt-3 space-y-3">
		{#each data.orders.items as o (o.id)}
			<li class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
				<div class="flex flex-wrap items-start justify-between gap-4">
					<div class="min-w-0 space-y-1">
						<div class="flex flex-wrap items-center gap-2">
							<span class="font-mono text-sm font-semibold">#{o.id.slice(0, 8).toUpperCase()}</span>
							<OrderStatusBadge status={o.status} />
							<span class="text-xs text-slate-400">{formatDateTime(o.submitted_at ?? o.created_at)}</span>
						</div>
						<p class="text-sm">
							<strong>{o.user_full_name}</strong>
							{#if o.user_phone}<a href="tel:{o.user_phone}" class="ml-2 font-mono text-slate-500">{o.user_phone}</a>{/if}
						</p>
						<ul class="text-sm text-slate-600">
							{#each o.items as item (item.course_id ?? item.ebook_id ?? item.title)}
								<li>{item.kind === 'course' ? 'វគ្គ' : 'សៀវភៅ'} · {item.title} — {formatPrice(item.price_cents)}</li>
							{/each}
						</ul>
						{#if o.admin_note}<p class="text-sm text-red-700">កំណត់ចំណាំ៖ {o.admin_note}</p>{/if}
					</div>
					<div class="text-right">
						<p class="text-xl font-bold text-brand-600">{formatPrice(o.total_cents)}</p>
						<p class="text-xs text-slate-500">{o.payment_method_code ?? '—'}</p>
						{#if o.payment_ref}
							<p class="mt-1 rounded bg-slate-100 px-2 py-0.5 font-mono text-sm select-all">{o.payment_ref}</p>
						{/if}
					</div>
				</div>

				{#if form?.id === o.id && form.code}
					<p class="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
				{/if}

				{#if canApprove(o.status)}
					<div class="mt-3 flex flex-wrap items-center gap-2 border-t border-slate-100 pt-3">
						<form method="POST" action="?/approve" use:enhance>
							<input type="hidden" name="id" value={o.id} />
							<button class="rounded-lg bg-green-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-green-700">អនុម័ត</button>
						</form>
						{#if o.status === 'awaiting_approval'}
							<form method="POST" action="?/reject" use:enhance class="flex flex-1 gap-2">
								<input type="hidden" name="id" value={o.id} />
								<input
									name="note"
									required
									maxlength="500"
									placeholder="មូលហេតុបដិសេធ (អ្នកទិញនឹងឃើញ)"
									class="min-w-0 flex-1 rounded-lg border border-slate-200 px-3 py-1.5 text-sm outline-none focus:border-red-500"
								/>
								<button class="rounded-lg border border-red-600 px-4 py-1.5 text-sm font-semibold text-red-600 hover:bg-red-50">បដិសេធ</button>
							</form>
						{/if}
					</div>
				{/if}
			</li>
		{/each}
	</ul>
	<Pagination page={data.orders.page} perPage={data.orders.per_page} total={data.orders.total} />
{/if}
