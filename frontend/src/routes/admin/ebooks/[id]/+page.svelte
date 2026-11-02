<script lang="ts">
	import { enhance } from '$app/forms';
	import EbookForm from '$lib/components/admin/EbookForm.svelte';
	import { confirmClick, dangerButton, secondaryButton } from '$lib/admin-ui';
	import { errorMessage } from '$lib/errors';
	import { formatDateTime } from '$lib/format';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const ebook = $derived(data.ebook);
</script>

<svelte:head><title>{ebook.title} · Admin</title></svelte:head>

<div class="flex flex-wrap items-start justify-between gap-4">
	<div>
		<a href="/admin/ebooks" class="text-sm text-brand-600 hover:underline">← សៀវភៅ</a>
		<h1 class="text-2xl font-bold">{ebook.title}</h1>
		<p class="text-xs text-slate-500">កែចុងក្រោយ {formatDateTime(ebook.updated_at)}</p>
	</div>
	<div class="flex gap-2">
		{#if ebook.published}
			<a href="/ebooks/{ebook.slug}" target="_blank" class={secondaryButton}>មើលទំព័រ ↗</a>
		{/if}
		<form method="POST" action="?/delete" use:enhance>
			<button class={dangerButton} onclick={confirmClick('លុបសៀវភៅនេះ? មិនអាចត្រឡប់វិញបានទេ។')}>លុប</button>
		</form>
	</div>
</div>

{#if form?.code}
	<p class="mt-3 rounded-lg bg-red-50 px-3 py-2 text-sm text-red-700" role="alert">{errorMessage(form.code)}</p>
{:else if form?.saved}
	<p class="mt-3 rounded-lg bg-green-50 px-3 py-2 text-sm text-green-700" role="status">បានរក្សាទុក</p>
{/if}

<section class="mt-4 rounded-xl border border-slate-200 bg-white p-5">
	{#key ebook.updated_at}
		<EbookForm {ebook} categories={data.categories} library={data.documents} action="?/save" />
	{/key}
</section>
