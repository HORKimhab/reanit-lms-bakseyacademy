<script lang="ts">
	import CourseForm from '$lib/components/admin/CourseForm.svelte';
	import { errorMessage } from '$lib/errors';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<svelte:head><title>វគ្គសិក្សាថ្មី · Admin</title></svelte:head>

<a href="/admin/courses" class="text-sm text-brand-600 hover:underline">← វគ្គសិក្សា</a>
<h1 class="mt-1 mb-4 text-2xl font-bold">វគ្គសិក្សាថ្មី</h1>
{#if !data.instructors.length}
	<p class="mb-4 rounded-lg bg-amber-50 px-3 py-2 text-sm text-amber-800">
		សូម<a href="/admin/instructors" class="underline">បន្ថែមគ្រូ</a>ជាមុនសិន។
	</p>
{/if}
{#if form?.code}
	<p class="mb-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{/if}
<div class="rounded-xl border border-slate-200 bg-white p-5">
	<CourseForm instructors={data.instructors} categories={data.categories} action="" />
</div>
