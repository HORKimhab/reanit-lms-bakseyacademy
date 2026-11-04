<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import { inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import { formatDateTime, formatNumber, formatPrice } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<svelte:head><title>កញ្ចប់ · Admin</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">កញ្ចប់</h1>
	<form method="GET" class="flex gap-2">
		<input name="q" type="search" value={data.q} placeholder="ចំណងជើង ឬ slug" class="{inputClass} w-64" />
		<button class={secondaryButton}>ស្វែងរក</button>
	</form>
</div>
<p class="text-sm text-slate-500">ទិញកញ្ចប់ = ទទួលបានគ្រប់វគ្គ និងសៀវភៅក្នុងនោះ។ កញ្ចប់ទទេមិនបង្ហាញ និងមិនលក់ទេ។</p>

<form method="POST" action="?/create" use:enhance class="mt-4 flex flex-wrap gap-2">
	<input name="title" required placeholder="ចំណងជើង" class="{inputClass} w-64" />
	<input name="slug" required placeholder="slug" class="{inputClass} w-48 font-mono" />
	<input name="price" required inputmode="decimal" placeholder="តម្លៃ $" class="{inputClass} w-28" />
	<button class={primaryButton}>+ កញ្ចប់ថ្មី</button>
</form>
<RowStatus {form} id="new" />

<div class="mt-4 overflow-x-auto rounded-xl border border-slate-200 bg-white">
	<table class="w-full text-sm">
		<thead class="bg-slate-50 text-left text-slate-500">
			<tr><th class="px-4 py-2 font-medium">ចំណងជើង</th><th class="px-4 py-2 font-medium">មុខ</th><th class="px-4 py-2 font-medium">តម្លៃ</th><th class="px-4 py-2 font-medium">លក់</th><th class="px-4 py-2 font-medium">កែចុងក្រោយ</th></tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each data.bundles.items as b (b.id)}
				<tr class="hover:bg-slate-50">
					<td class="px-4 py-2">
						<a href="/admin/bundles/{b.id}" class="font-medium text-brand-600 hover:underline">{b.title}</a>
						{#if !b.published}<span class="ml-1 rounded bg-slate-100 px-1.5 text-xs text-slate-600">ព្រាង</span>{/if}
					</td>
					<td class="px-4 py-2">{formatNumber(b.item_count)}</td>
					<td class="px-4 py-2">{formatPrice(b.sale_price_cents ?? b.price_cents)}</td>
					<td class="px-4 py-2">{formatNumber(b.sales)}</td>
					<td class="px-4 py-2 text-slate-500">{formatDateTime(b.updated_at)}</td>
				</tr>
			{:else}
				<tr><td colspan="5" class="px-4 py-6 text-center text-slate-500">មិនទាន់មានកញ្ចប់</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<div class="mt-6"><Pagination page={data.bundles.page} perPage={data.bundles.per_page} total={data.bundles.total} /></div>
