<script lang="ts">
	import { enhance } from '$app/forms';
	import Pagination from '$lib/components/Pagination.svelte';
	import { inputClass, primaryButton, secondaryButton } from '$lib/admin-ui';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime, formatPrice, formatNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<svelte:head><title>វគ្គសិក្សា · Admin</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">វគ្គសិក្សា</h1>
	<div class="flex gap-2">
		<form method="GET" class="flex gap-2">
			<input name="q" type="search" value={data.q} placeholder="ចំណងជើង slug ឬគ្រូ" class="{inputClass} w-64" />
			<button class={secondaryButton}>ស្វែងរក</button>
		</form>
		<a href="/admin/courses/new" class={primaryButton}>+ វគ្គសិក្សាថ្មី</a>
	</div>
</div>

{#if form?.code}
	<p class="mt-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{/if}

<p class="mt-4 text-sm text-slate-500">{formatNumber(data.courses.total)} វគ្គសិក្សា</p>

<div class="mt-2 overflow-x-auto rounded-xl border border-slate-200 bg-white">
	<table class="w-full text-sm">
		<thead class="bg-slate-50 text-left text-slate-500">
			<tr>
				<th class="px-4 py-2 font-medium">ចំណងជើង</th>
				<th class="px-4 py-2 font-medium">គ្រូ</th>
				<th class="px-4 py-2 font-medium">តម្លៃ</th>
				<th class="px-4 py-2 font-medium">មេរៀន</th>
				<th class="px-4 py-2 font-medium">អ្នកសិក្សា</th>
				<th class="px-4 py-2 font-medium">កែចុងក្រោយ</th>
				<th class="px-4 py-2 font-medium">ស្ថានភាព</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each data.courses.items as c (c.id)}
				<tr>
					<td class="px-4 py-2">
						<a href="/admin/courses/{c.id}" class="font-semibold hover:text-brand-600">{c.title}</a>
						<p class="font-mono text-xs text-slate-400">{c.slug}</p>
					</td>
					<td class="px-4 py-2">{c.instructor_name}</td>
					<td class="px-4 py-2">
						{formatPrice(c.sale_price_cents ?? c.price_cents)}
						{#if c.sale_price_cents !== null}<s class="text-xs text-slate-400">{formatPrice(c.price_cents)}</s>{/if}
					</td>
					<td class="px-4 py-2">{formatNumber(c.lesson_count)}</td>
					<td class="px-4 py-2">{formatNumber(c.enrollment_count)}</td>
					<td class="px-4 py-2 text-xs text-slate-500">{formatDateTime(c.updated_at)}</td>
					<td class="px-4 py-2">
						<form method="POST" action="?/publish" use:enhance>
							<input type="hidden" name="id" value={c.id} />
							<input type="hidden" name="published" value={String(!c.published)} />
							<button
								class="rounded-full px-3 py-0.5 text-xs font-semibold {c.published
									? 'bg-green-100 text-green-700 hover:bg-green-200'
									: 'bg-slate-100 text-slate-600 hover:bg-slate-200'}"
								title={c.published ? 'ចុចដើម្បីបិទការផ្សាយ' : 'ចុចដើម្បីផ្សាយ'}
							>
								{c.published ? 'បានផ្សាយ' : 'ព្រាង'}
							</button>
						</form>
					</td>
				</tr>
			{:else}
				<tr><td colspan="7" class="px-4 py-8 text-center text-slate-500">មិនមានវគ្គសិក្សាទេ</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<Pagination page={data.courses.page} perPage={data.courses.per_page} total={data.courses.total} />
