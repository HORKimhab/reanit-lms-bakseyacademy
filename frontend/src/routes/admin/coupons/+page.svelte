<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import { confirmClick, dangerButton, dollarsInput, inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import { formatNumber, formatPrice } from '$lib/format';
	import type { AdminCoupon } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const keep = () => ({ update }: { update: (o?: { reset?: boolean }) => Promise<void> }) => update({ reset: false });

	/** ISO → `YYYY-MM-DD` in Phnom Penh time, for `<input type="date">`. */
	const dateInput = (iso: string | null) =>
		iso ? new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Phnom_Penh' }).format(new Date(iso)) : '';
	const expired = (c: AdminCoupon) => c.expires_at !== null && new Date(c.expires_at) <= new Date();
</script>

<svelte:head><title>កូដបញ្ចុះតម្លៃ · Admin</title></svelte:head>

<h1 class="text-2xl font-bold">កូដបញ្ចុះតម្លៃ</h1>
<p class="text-sm text-slate-500">
	អ្នកទិញវាយកូដនៅក្នុងកន្ត្រក។ ភាគរយ (១–១០០) ឬចំនួនទឹកប្រាក់ ($) ដកចេញពីសរុប។ ថ្ងៃផុតកំណត់ = ចុងថ្ងៃនោះ។ ចំនួនប្រើ = ការបញ្ជាទិញមិនបានបោះបង់។
</p>

{#snippet fields(c: AdminCoupon | null)}
	<input name="code" required value={c?.code ?? ''} placeholder="CODE" class="{inputClass} w-36 font-mono uppercase" />
	<select name="kind" class="{inputClass} w-20" aria-label="ប្រភេទ">
		<option value="percent" selected={c ? c.percent_off !== null : true}>%</option>
		<option value="amount" selected={c ? c.amount_off_cents !== null : false}>$</option>
	</select>
	<input
		name="value"
		required
		value={c ? (c.percent_off ?? dollarsInput(c.amount_off_cents)) : ''}
		inputmode="decimal"
		placeholder="10"
		class="{inputClass} w-20"
		aria-label="តម្លៃ"
	/>
	<label class="flex items-center gap-1 text-sm">ផុតកំណត់ <input name="expires" type="date" value={dateInput(c?.expires_at ?? null)} class="{inputClass} w-40" /></label>
	<input name="max_uses" value={c?.max_uses ?? ''} inputmode="numeric" placeholder="∞" class="{inputClass} w-16" aria-label="ប្រើបានអតិបរមា" title="ប្រើបានអតិបរមា" />
	<label class="flex items-center gap-1 text-sm"><input type="checkbox" name="active" checked={c?.active ?? true} class="accent-brand-600" /> សកម្ម</label>
{/snippet}

<ul class="mt-4 space-y-2">
	{#each data.coupons as c (c.id)}
		<li class="rounded-xl border border-slate-200 bg-white p-3">
			<div class="flex flex-wrap items-center gap-2">
				<form method="POST" action="?/update" use:enhance={keep} class="flex flex-1 flex-wrap items-center gap-2">
					<input type="hidden" name="id" value={c.id} />
					{@render fields(c)}
					<button class={secondaryButton}>រក្សាទុក</button>
				</form>
				<form method="POST" action="?/delete" use:enhance>
					<input type="hidden" name="id" value={c.id} />
					<button class={dangerButton} onclick={confirmClick(`លុបកូដ "${c.code}"?`)}>លុប</button>
				</form>
			</div>
			<p class="mt-2 text-xs text-slate-500">
				ប្រើ {formatNumber(c.uses)}{c.max_uses ? ` / ${formatNumber(c.max_uses)}` : ''} ដង · បញ្ចុះសរុប {formatPrice(c.discount_given_cents)}
				{#if !c.active}<span class="ml-2 rounded bg-slate-100 px-1.5 text-slate-600">បិទ</span>{/if}
				{#if expired(c)}<span class="ml-2 rounded bg-red-50 px-1.5 text-red-700">ផុតកំណត់</span>{/if}
			</p>
			<RowStatus {form} id={c.id} />
		</li>
	{:else}
		<li class="text-sm text-slate-500">មិនទាន់មានកូដ</li>
	{/each}
</ul>

<h2 class="mt-8 font-semibold">កូដថ្មី</h2>
<form method="POST" action="?/create" use:enhance class="mt-2 flex flex-wrap items-center gap-2">
	{@render fields(null)}
	<button class={primaryButton}>+ បន្ថែម</button>
</form>
<RowStatus {form} id="new" />
