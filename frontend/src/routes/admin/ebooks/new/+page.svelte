<script lang="ts">
	import EbookForm from '$lib/components/admin/EbookForm.svelte';
	import { errorMessage } from '$lib/errors';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<svelte:head><title>សៀវភៅថ្មី · Admin</title></svelte:head>

<a href="/admin/ebooks" class="text-sm text-brand-600 hover:underline">← សៀវភៅ</a>
<h1 class="mt-1 mb-4 text-2xl font-bold">សៀវភៅថ្មី</h1>
{#if form?.code}
	<p class="mb-4 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{/if}
<div class="rounded-xl border border-slate-200 bg-white p-5">
	<EbookForm categories={data.categories} action="" />
</div>
