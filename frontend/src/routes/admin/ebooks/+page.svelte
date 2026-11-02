<script lang="ts">
	import { enhance } from '$app/forms';
	import Pagination from '$lib/components/Pagination.svelte';
	import { inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime, formatPrice, khmerNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<svelte:head><title>សៀវភៅ · Admin</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">សៀវភៅ</h1>
	<div class="flex gap-2">
		<form method="GET" class="flex gap-2">
			<input name="q" type="search" value={data.q} placeholder="ចំណងជើង slug ឬអ្នកនិពន្ធ" class="{inputClass} w-64" />
			<button class={secondaryButton}>ស្វែងរក</button>
		</form>
		<a href="/admin/ebooks/new" class={primaryButton}>+ សៀវភៅថ្មី</a>
	</div>
</div>

{#if form?.code}
	<p class="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{/if}

<p class="mt-4 text-sm text-slate-500">{khmerNumber(data.ebooks.total)} សៀវភៅ</p>

<div class="mt-2 overflow-x-auto rounded-xl border border-slate-200 bg-white">
	<table class="w-full text-sm">
		<thead class="bg-slate-50 text-left text-slate-500">
			<tr>
				<th class="px-4 py-2 font-medium">ចំណងជើង</th>
				<th class="px-4 py-2 font-medium">អ្នកនិពន្ធ</th>
				<th class="px-4 py-2 font-medium">តម្លៃ</th>
				<th class="px-4 py-2 font-medium">ឯកសារ</th>
				<th class="px-4 py-2 font-medium">អ្នកទិញ</th>
				<th class="px-4 py-2 font-medium">កែចុងក្រោយ</th>
				<th class="px-4 py-2 font-medium">ស្ថានភាព</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each data.ebooks.items as e (e.id)}
				<tr>
					<td class="px-4 py-2">
						<a href="/admin/ebooks/{e.id}" class="font-semibold hover:text-brand-600">{e.title}</a>
						<p class="font-mono text-xs text-slate-400">{e.slug}</p>
					</td>
					<td class="px-4 py-2">{e.author_name}</td>
					<td class="px-4 py-2">
						{formatPrice(e.sale_price_cents ?? e.price_cents)}
						{#if e.sale_price_cents !== null}<s class="text-xs text-slate-400">{formatPrice(e.price_cents)}</s>{/if}
					</td>
					<td class="px-4 py-2">{#if e.has_file}<span class="text-green-700">✓</span>{:else}<span class="text-amber-600">—</span>{/if}</td>
					<td class="px-4 py-2">{khmerNumber(e.purchase_count)}</td>
					<td class="px-4 py-2 text-xs text-slate-500">{formatDateTime(e.updated_at)}</td>
					<td class="px-4 py-2">
						<form method="POST" action="?/publish" use:enhance>
							<input type="hidden" name="id" value={e.id} />
							<input type="hidden" name="published" value={String(!e.published)} />
							<button
								class="rounded-full px-3 py-0.5 text-xs font-semibold {e.published
									? 'bg-green-100 text-green-700 hover:bg-green-200'
									: 'bg-slate-100 text-slate-600 hover:bg-slate-200'}"
								title={e.published ? 'ចុចដើម្បីបិទការផ្សាយ' : 'ចុចដើម្បីផ្សាយ'}
							>
								{e.published ? 'បានផ្សាយ' : 'ព្រាង'}
							</button>
						</form>
					</td>
				</tr>
			{:else}
				<tr><td colspan="7" class="px-4 py-8 text-center text-slate-500">មិនមានសៀវភៅទេ</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<Pagination page={data.ebooks.page} perPage={data.ebooks.per_page} total={data.ebooks.total} />
