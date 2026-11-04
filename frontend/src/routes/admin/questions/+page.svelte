<script lang="ts">
	import { enhance } from '$app/forms';
	import RowStatus from '$lib/components/admin/RowStatus.svelte';
	import Pagination from '$lib/components/Pagination.svelte';
	import { inputClass, primaryButton } from '$lib/admin-ui';
	import { formatDateTime, formatNumber } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const q = $derived(data.questions);
</script>

<svelte:head><title>សំណួរ · Admin</title></svelte:head>

<div class="flex flex-wrap items-center justify-between gap-4">
	<h1 class="text-2xl font-bold">សំណួររបស់សិស្ស</h1>
	<nav class="flex gap-1 rounded-lg border border-slate-200 bg-white p-1 text-sm">
		<a href="/admin/questions" class="rounded-md px-3 py-1 {data.all ? 'hover:bg-slate-100' : 'bg-brand-600 text-white'}">មិនទាន់ឆ្លើយ</a>
		<a href="/admin/questions?all=1" class="rounded-md px-3 py-1 {data.all ? 'bg-brand-600 text-white' : 'hover:bg-slate-100'}">ទាំងអស់</a>
	</nav>
</div>
<p class="text-sm text-slate-500">គ្រូនៃវគ្គ (គណនីភ្ជាប់នឹងគ្រូ) ក៏អាចឆ្លើយនៅទំព័រមេរៀនបានដែរ។</p>

{#if q.items.length === 0}
	<p class="mt-8 text-center text-slate-500">{data.all ? 'មិនទាន់មានសំណួរ' : 'បានឆ្លើយគ្រប់សំណួរហើយ ✓'}</p>
{:else}
	<ul class="mt-4 space-y-3">
		{#each q.items as item (item.id)}
			<li class="rounded-xl border border-slate-200 bg-white p-4">
				<p class="text-xs text-slate-500">
					<a href="/courses/{item.course_slug}/learn/{item.lesson_id}" class="text-brand-600 hover:underline">{item.course_title} › {item.lesson_title}</a>
					· {item.user_name} · {formatDateTime(item.created_at)}
					{#if item.answer_count}· ចម្លើយ {formatNumber(item.answer_count)}{/if}
				</p>
				<p class="mt-1 whitespace-pre-line">{item.body}</p>
				<form method="POST" action="?/answer" use:enhance class="mt-3 flex gap-2">
					<input type="hidden" name="id" value={item.id} />
					<input name="body" required maxlength="4000" placeholder="សរសេរចម្លើយ…" class="{inputClass} flex-1" />
					<button class={primaryButton}>ឆ្លើយ</button>
				</form>
				<RowStatus {form} id={item.id} />
			</li>
		{/each}
	</ul>
	<div class="mt-6"><Pagination page={q.page} perPage={q.per_page} total={q.total} /></div>
{/if}
