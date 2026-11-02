<script lang="ts">
	import { m } from '$lib/paraglide/messages.js';
	import { localized } from '$lib/i18n';
	import { enhance } from '$app/forms';
	import OrderStatusBadge from '$lib/components/OrderStatusBadge.svelte';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime, formatPrice } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const order = $derived(data.order);
	const method = $derived(order.payment_method);
	const canSubmit = $derived(order.status === 'pending_payment' || order.status === 'rejected');
	const shortId = $derived(order.id.slice(0, 8).toUpperCase());
</script>

<svelte:head><title>{m.order_title({ id: shortId })} · Reanit Academy</title></svelte:head>

<div class="mx-auto max-w-3xl">
	<div class="flex flex-wrap items-center gap-3">
		<h1 class="text-3xl font-bold">{m.order_title({ id: shortId })}</h1>
		<OrderStatusBadge status={order.status} />
	</div>
	<p class="mt-1 text-sm text-slate-500">{formatDateTime(order.created_at)}</p>

	{#if form?.code}
		<p class="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
	{/if}

	{#if order.status === 'awaiting_approval'}
		<p class="mt-6 rounded-xl bg-sky-50 p-4 text-sky-800">
			{m.order_ref_received_before()} <strong>{order.payment_ref}</strong>{m.order_ref_received_after()}
		</p>
	{:else if order.status === 'paid'}
		<p class="mt-6 rounded-xl bg-green-50 p-4 text-green-800">
			{m.order_paid_message()}
			<a href="/account/library" class="font-semibold underline">{m.order_go_to_library()}</a>
		</p>
	{:else if order.status === 'rejected'}
		<div class="mt-6 rounded-xl bg-red-50 p-4 text-red-800">
			<p class="font-semibold">{m.order_rejected_title()}</p>
			{#if order.admin_note}<p class="mt-1">{order.admin_note}</p>{/if}
			<p class="mt-1 text-sm">{m.order_rejected_hint()}</p>
		</div>
	{:else if order.status === 'cancelled'}
		<p class="mt-6 rounded-xl bg-slate-100 p-4 text-slate-600">{m.order_cancelled_message()}</p>
	{/if}

	<div class="mt-6 grid gap-6 {canSubmit && method ? 'md:grid-cols-2' : ''}">
		{#if canSubmit && method}
			<section class="rounded-2xl border border-slate-200 bg-white p-5 text-center shadow-sm">
				<h2 class="font-semibold">{m.order_scan_to_pay({ method: localized(method.name_km, method.name_en) })}</h2>
				{#if method.qr_image_url}
					<img src={method.qr_image_url} alt="QR {method.name_en}" class="mx-auto mt-4 w-64 max-w-full" />
				{/if}
				<p class="mt-4 text-3xl font-bold text-brand-600">{formatPrice(order.total_cents)}</p>
				{#if method.account_name}<p class="mt-2 text-sm">{method.account_name}</p>{/if}
				{#if method.account_number}<p class="font-mono text-sm text-slate-500">{method.account_number}</p>{/if}
			</section>
		{/if}

		<div class="space-y-6">
			{#if canSubmit && method}
				<section class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
					<ol class="list-decimal space-y-1 pl-5 text-sm text-slate-700">
						<li>{m.order_step_scan()}</li>
						<li>{m.order_step_pay_before()} <strong>{formatPrice(order.total_cents)}</strong> {m.order_step_pay_after()}</li>
						<li>{m.order_step_enter_ref()}</li>
					</ol>
					<form method="POST" action="?/submit" use:enhance class="mt-4 space-y-3">
						<label class="block">
							<span class="mb-1 block text-sm font-medium">{m.order_ref_label()}</span>
							<input
								name="payment_ref"
								required
								minlength="3"
								maxlength="100"
								value={form?.payment_ref ?? (order.status === 'rejected' ? (order.payment_ref ?? '') : '')}
								class="w-full rounded-lg border border-slate-200 px-3 py-2 font-mono outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20"
							/>
						</label>
						<button class="w-full rounded-lg bg-brand-600 py-2.5 font-semibold text-white hover:bg-brand-700">{m.order_submit_paid()}</button>
					</form>
					{#if order.status === 'pending_payment'}
						<form method="POST" action="?/cancel" use:enhance class="mt-3 text-center">
							<button class="text-sm text-slate-500 hover:text-red-600 hover:underline">{m.order_cancel()}</button>
						</form>
					{/if}
				</section>
			{/if}

			<section class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
				<h2 class="font-semibold">{m.order_items()}</h2>
				<ul class="mt-3 divide-y divide-slate-100 text-sm">
					{#each order.items as item (item.course_id ?? item.ebook_id ?? item.title)}
						<li class="flex justify-between gap-4 py-2">
							<span><span class="text-slate-400">{item.kind === 'course' ? m.kind_course() : m.kind_ebook()} ·</span> {item.title}</span>
							<span class="whitespace-nowrap">{formatPrice(item.price_cents)}</span>
						</li>
					{/each}
				</ul>
				<p class="mt-3 flex justify-between border-t border-slate-200 pt-3 font-semibold">
					<span>{m.total()}</span><span>{formatPrice(order.total_cents)}</span>
				</p>
				{#if method && !canSubmit}<p class="mt-2 text-sm text-slate-500">{m.order_paid_via({ method: localized(method.name_km, method.name_en) })}</p>{/if}
			</section>
		</div>
	</div>
</div>
