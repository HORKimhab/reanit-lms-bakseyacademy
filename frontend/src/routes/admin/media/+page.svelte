<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import Pagination from '$lib/components/Pagination.svelte';
	import MediaField from '$lib/components/admin/MediaField.svelte';
	import { inputClass, secondaryButton } from '$lib/admin-ui';
	import { formatClock, formatDateTime, formatNumber } from '$lib/format';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const size = (b: number | null) => (b === null ? '—' : b >= 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.ceil(b / 1024)} KB`);
</script>

<svelte:head><title>ឯកសារ · Admin</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">ឯកសារ (Telegram)</h1>
	<form method="GET" class="flex gap-2">
		<input name="q" type="search" value={data.q} placeholder="ឈ្មោះឯកសារ" class="{inputClass} w-64" />
		<button class={secondaryButton}>ស្វែងរក</button>
	</form>
</div>
<p class="mt-1 text-sm text-slate-500">
	ផ្ទុកឡើងនៅទីនេះ ឬបង្ហោះវីដេអូ / ឯកសារក្នុង channel ផ្ទុកទុក (webhook នាំចូលដោយស្វ័យប្រវត្តិ)។ ភ្ជាប់ទៅមេរៀន ឬសៀវភៅពីទំព័រកែសម្រួលរបស់វា។
</p>
<div class="mt-3"><MediaField onuploaded={() => invalidateAll()} /></div>

<p class="mt-4 text-sm text-slate-500">{formatNumber(data.media.total)} ឯកសារ</p>
<div class="mt-2 overflow-x-auto rounded-xl border border-slate-200 bg-white">
	<table class="w-full text-sm">
		<thead class="bg-slate-50 text-left text-slate-500">
			<tr>
				<th class="px-4 py-2 font-medium">ឈ្មោះ</th>
				<th class="px-4 py-2 font-medium">ប្រភេទ</th>
				<th class="px-4 py-2 font-medium">ទំហំ</th>
				<th class="px-4 py-2 font-medium">រយៈពេល</th>
				<th class="px-4 py-2 font-medium">បានបន្ថែម</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-slate-100">
			{#each data.media.items as f (f.id)}
				<tr>
					<td class="px-4 py-2">
						{#if f.preview_url}<a href={f.preview_url} target="_blank" class="hover:text-brand-600">{f.file_name ?? '—'} ↗</a>{:else}{f.file_name ?? '—'}{/if}
						<p class="font-mono text-xs text-slate-400">{f.id}</p>
					</td>
					<td class="px-4 py-2 text-xs">{f.mime_type ?? '—'}</td>
					<td class="px-4 py-2">{size(f.size_bytes)}</td>
					<td class="px-4 py-2">{f.duration_seconds ? formatClock(f.duration_seconds) : '—'}</td>
					<td class="px-4 py-2 text-xs text-slate-500">{formatDateTime(f.created_at)}</td>
				</tr>
			{:else}
				<tr><td colspan="5" class="px-4 py-8 text-center text-slate-500">មិនមានឯកសារទេ</td></tr>
			{/each}
		</tbody>
	</table>
</div>
<Pagination page={data.media.page} perPage={data.media.per_page} total={data.media.total} />
