<script lang="ts">
	import Pagination from '$lib/components/Pagination.svelte';
	import { inputClass, secondaryButton } from '$lib/admin-ui';
	import { formatDateTime, formatNumber } from '$lib/format';
	import { roleLabels } from './roles';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head><title>អ្នកប្រើ · Admin</title></svelte:head>

<h1 class="text-2xl font-bold">អ្នកប្រើប្រាស់</h1>

<form method="GET" class="mt-4 flex flex-wrap gap-2">
	<input name="q" value={data.q} placeholder="ឈ្មោះ ទូរស័ព្ទ អ៊ីមែល…" class="{inputClass} max-w-xs" />
	<select name="role" class="{inputClass} w-40">
		<option value="">តួនាទីទាំងអស់</option>
		{#each Object.entries(roleLabels) as [value, label] (value)}
			<option {value} selected={data.role === value}>{label}</option>
		{/each}
	</select>
	<button class={secondaryButton}>ស្វែងរក</button>
</form>

<div class="mt-4 overflow-x-auto rounded-xl border border-slate-200 bg-white">
	<table class="w-full text-sm">
		<thead class="bg-slate-50 text-left text-slate-600">
			<tr><th class="px-4 py-2">ឈ្មោះ</th><th class="px-4 py-2">ទូរស័ព្ទ / អ៊ីមែល</th><th class="px-4 py-2">តួនាទី</th><th class="px-4 py-2">វគ្គ / សៀវភៅ</th><th class="px-4 py-2">ចុះឈ្មោះ</th></tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each data.users.items as u (u.id)}
				<tr class="hover:bg-slate-50">
					<td class="px-4 py-2"><a href="/admin/users/{u.id}" class="font-medium text-brand-600 hover:underline">{u.full_name}</a></td>
					<td class="px-4 py-2 text-slate-600">{u.phone ?? ''}{u.phone && u.email ? ' · ' : ''}{u.email ?? ''}</td>
					<td class="px-4 py-2">{roleLabels[u.role] ?? u.role}</td>
					<td class="px-4 py-2">{formatNumber(u.course_count)} / {formatNumber(u.ebook_count)}</td>
					<td class="px-4 py-2 text-slate-500">{formatDateTime(u.created_at)}</td>
				</tr>
			{:else}
				<tr><td colspan="5" class="px-4 py-6 text-center text-slate-500">រកមិនឃើញ</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<div class="mt-6"><Pagination page={data.users.page} perPage={data.users.per_page} total={data.users.total} /></div>
